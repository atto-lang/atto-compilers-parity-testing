import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,readdirSync,mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {HdlV3Adapter as Session} from './rtl-session.mjs';
import {pathToFileURL} from 'node:url';
const {compileNamespaceProvider,evaluateReference,emitSystemVerilog}=await import(pathToFileURL(process.env.ATTO_HDL_COMPILER).href);
const root=process.env.ATTO_V3_FIXTURE;
const read=p=>readFileSync(join(root,p),'utf8');
const oracle=JSON.parse(read('fixture.json'));
const pm=read('checkout/.ores.pkg.toml');
const files=new Map(readdirSync(join(root,'checkout/src')).sort().map(n=>['src/'+n,read('checkout/src/'+n)]));
const hash=s=>createHash('sha256').update(s,'utf8').digest('hex');
const pin=s=>hash([...s].sort(([a],[b])=>a<b?-1:a>b?1:0).map(([p,t])=>p+'\n'+hash(t)+'\n').join(''));
const baseline=pin(files);
const args=(name,source=files,manifest=read(name+'/.ores.pkg.toml'),consumer=read(name+'/src/main.ores'))=>({consumer,consumerManifest:manifest,providerManifest:pm,sources:source});
const repin=(name,source,consumer)=>args(name,source,read(name+'/.ores.pkg.toml').replace(baseline,pin(source)),consumer);
const guest=async artifact=>{
 const d=mkdtempSync(join(tmpdir(),'hdl-v3-guest-'));
 try{
  const rtl=join(d,'guest.sv'),exe=join(d,'sim');writeFileSync(rtl,artifact.code);
  const build=spawnSync('iverilog',['-g2012','-s','tb','-o',exe,rtl],{encoding:'utf8',timeout:10000});
  assert.equal(build.status,0,String(build.error??build.stderr));
  const p=spawnSync('vvp',[exe],{encoding:'utf8',timeout:5000});
  assert.equal(p.status,0,String(p.error??p.stderr));return p.stdout.trim().split('\n').filter(x=>/^[0-9]+$/.test(x));
 }finally{rmSync(d,{recursive:true,force:true})}
};
const counts=x=>[x.classified,x.classificationReused,x.compiled,x.artifactReused,x.relinked];

test('canonical V3 fixture: real provider RTL modules, cold warm edit clean and unchanged observer',async()=>{
 assert.equal(files.size,12);assert.equal(hash(pm),oracle.manifest_sha256);assert.equal(baseline,oracle.sources_sha256);
 const c=new Session(),o=new Session();
 const cold=c.build(args('client')),other=o.build(args('observer'));
 assert.deepEqual(await guest(cold),oracle.lanes.cold.billing);assert.deepEqual(await guest(other),oracle.lanes.cold.identity);
 assert.deepEqual(counts(cold),[12,0,4,0,1]);assert.deepEqual(counts(other),[12,0,3,0,1]);
 const warm=c.build(args('client'));assert.deepEqual(counts(warm),[0,12,0,4,0]);assert.equal(warm.code,cold.code);
 const changed=new Map(files);changed.set('src/unit0.ores',files.get('src/unit0.ores').replace('return 0','return 42'));
 assert.equal(pin(changed),oracle.lanes.body_edit.repinned_sources_sha256);
 assert.throws(()=>c.build(args('client',changed)),/source pin mismatch/);
 const edit=c.build(repin('client',changed)),observer=o.build(repin('observer',changed));
 assert.deepEqual(await guest(edit),oracle.lanes.body_edit.billing);assert.deepEqual(await guest(observer),oracle.lanes.body_edit.identity);
 assert.deepEqual(counts(edit),[1,11,1,3,1]);assert.deepEqual(counts(observer),[1,11,0,3,0]);
 assert.equal(observer.code,other.code);
 const clean=new Session().build(repin('client',changed));assert.equal(edit.code,clean.code);assert.deepEqual(await guest(clean),await guest(edit));
 const abi=new Map(changed);abi.set('src/unit0.ores',oracle.lanes.abi_edit.new_source);
 assert.equal(pin(abi),oracle.lanes.abi_edit.repinned_sources_sha256);
 assert.throws(()=>c.build(repin('client',abi)),/E_NAMESPACE_ARITY/);
 const recovered=c.build(repin('client',changed));assert.deepEqual(counts(recovered),[0,12,0,4,0]);assert.equal(recovered.code,edit.code);
 console.log('V3_HDL_RTL|guest=0/1/2/3->42/1/2/3|observer=4/5/6|compiler=4/0/1/0|classification=12/0/1|observer-byte-stable|clean-differential|abi|rollback');
});
test('reject unlisted, foreign, private, removed, forged and invalid compiler literal',async()=>{
 const c=new Session();const accepted=c.build(args('client'));
 const extra=new Map(files);extra.set('src/extra.ores','namespace billing;\npub fnc extra(): int { return 99; }\n');
 assert.throws(()=>c.build(repin('client',extra)),/inventory/);
 const missing=new Map(files);missing.delete('src/unit0.ores');
 assert.throws(()=>c.build(repin('client',missing)),/inventory/);
 const consumer=read('client/src/main.ores');
 for(const bad of [consumer.replace('provider.function0','provider.function4'),consumer.replace('provider.function0','provider.local0')])
  assert.throws(()=>c.build(repin('client',files,bad)),/E_NAMESPACE_SYMBOL/);
 const forged=consumer.replace('import namespace billing as provider from "checkout";','import * as provider from "pkg:namespace:checkout:billing";');
 assert.throws(()=>c.build(repin('client',files,forged)),/E_NAMESPACE_PROFILE/);
 const hidden=new Map(files);hidden.set('src/unit0.ores',files.get('src/unit0.ores').replace('pub fnc','fnc'));
 assert.throws(()=>c.build(repin('client',hidden)),/E_NAMESPACE_PROFILE/);
 const overflow=new Map(files);overflow.set('src/unit0.ores',files.get('src/unit0.ores').replace('return 0','return 9999999999999999999'));
 assert.throws(()=>c.build(repin('client',overflow)),/E_NAMESPACE_LITERAL/);
 const recovered=c.build(args('client'));assert.equal(recovered.code,accepted.code);assert.deepEqual(counts(recovered),[0,12,0,4,0]);assert.deepEqual(await guest(recovered),oracle.lanes.cold.billing);
 console.log('V3_HDL_NEGATIVES|unlisted|removed|foreign|unnamespaced|private|forged|compiler-overflow|rollback');
});

test('actual admitted constant AST, exact range and fail-closed profile',async()=>{
 const src='namespace billing;\npub fnc function0(): int { return 9223372036854775807; }\n';
 const {program,systemVerilog}=compileNamespaceProvider(src,'billing','function0');
 assert.equal(evaluateReference(program,{}),9223372036854775807n);
 assert.equal(emitSystemVerilog(program),systemVerilog);
 assert.throws(()=>{program.expression.value=0n;},TypeError);
 assert.throws(()=>emitSystemVerilog({...program}),e=>e.code==='HDL_UNVERIFIED_AST');
 const tb='module tb; wire [63:0] result; ores_function0 dut(.out_result(result)); initial begin #1; $display("%0d",result); $finish; end endmodule';
 assert.deepEqual(await guest({code:systemVerilog+'\n'+tb}),['9223372036854775807']);
 for(const literal of ['-1','9223372036854775808','01','1+2'])assert.throws(()=>compileNamespaceProvider(src.replace('9223372036854775807',literal),'billing','function0'),e=>e.code==='E_NAMESPACE_LITERAL');
 assert.throws(()=>compileNamespaceProvider(src.replace('()','(int x)'),'billing','function0'),e=>e.code==='E_NAMESPACE_ARITY');
 assert.throws(()=>compileNamespaceProvider(src,'identity','function0'),e=>e.code==='E_NAMESPACE_PROFILE');
 console.log('HDL_CONSTANT_BOUNDARY|signed-max|negative-denied|overflow-denied|immutable-admission');
});
