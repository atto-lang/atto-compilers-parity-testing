import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,readdirSync,mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {CheckedHdlV3Adapter as Session} from './checked-rtl-session.mjs';
import {pathToFileURL} from 'node:url';
const {compileCheckedNamespaceProvider,evaluateReference,emitSystemVerilog}=await import(pathToFileURL(process.env.ATTO_HDL_COMPILER).href);
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
  assert.equal(p.status,0,String(p.error??p.stderr));return p.stdout.trim().split('\n').filter(x=>!/^.*guest\.sv:\d+: \$finish called at \d+ \(.*\)$/.test(x));
 }finally{rmSync(d,{recursive:true,force:true})}
};
const counts=x=>[x.classified,x.classificationReused,x.compiled,x.artifactReused,x.relinked];

test('parameterized checked RTL and compiler-built argument helpers',async()=>{
 const c=new Session(),o=new Session(),original=read('client/src/main.ores');
 const consumer=original.replace('provider.function0()','provider.function0(5+5,4)');
 const changed=new Map(files);changed.set('src/unit0.ores','namespace billing;\npub fnc function0(int x, int y): int { return x * y + 2; }\n');
 const cold=c.build(repin('client',changed,consumer)),observer=o.build(repin('observer',changed));
 assert.deepEqual(await guest(cold),['42','1','2','3']);assert.deepEqual(await guest(observer),['4','5','6']);
 assert.deepEqual(counts(cold),[12,0,4,0,1]);assert.equal(cold.argumentCompiled,2);
 const warm=c.build(repin('client',changed,consumer));assert.deepEqual(counts(warm),[0,12,0,4,0]);assert.equal(warm.argumentReused,2);
 const argEdit=c.build(repin('client',changed,consumer.replace('5+5','5+6')));assert.deepEqual(await guest(argEdit),['46','1','2','3']);
 assert.deepEqual(counts(argEdit),[0,12,0,4,1]);assert.equal(argEdit.argumentCompiled,1);assert.equal(argEdit.argumentReused,1);
 const body=new Map(changed);body.set('src/unit0.ores',changed.get('src/unit0.ores').replace('+ 2','+ 3'));
 const edit=c.build(repin('client',body,consumer));assert.deepEqual(await guest(edit),['43','1','2','3']);assert.deepEqual(counts(edit),[1,11,1,3,1]);
 const untouched=o.build(repin('observer',body));assert.equal(untouched.code,observer.code);assert.deepEqual(counts(untouched),[1,11,0,3,0]);
 assert.equal(new Session().build(repin('client',body,consumer)).code,edit.code);
 assert.throws(()=>c.build(repin('client',body,original)),/E_NAMESPACE_ARITY/);
 assert.throws(()=>c.build(repin('client',body,consumer.replace('5+5','stdio.println(5)'))),/E_NAMESPACE_PROFILE/);
 const recovered=c.build(repin('client',body,consumer));assert.equal(recovered.code,edit.code);assert.deepEqual(counts(recovered),[0,12,0,4,0]);assert.equal(recovered.argumentReused,2);
 console.log('V3_HDL_PARAMETERIZED|guest=42/46/43|checked-rtl-abi|compiler-built-argument-modules|argument-only-provider-reuse|observer-byte-stable|rollback');
});

test('checked signed RTL boundaries and intermediate overflow fault',()=>{
 const src='namespace billing;\npub fnc function0(int x, int y): int { return x * y + 2; }\n';
 const {program,systemVerilog}=compileCheckedNamespaceProvider(src,'billing','function0',2);
 assert.equal(evaluateReference(program,{x:10n,y:4n}),42n);
 assert.throws(()=>evaluateReference(program,{x:9223372036854775807n,y:2n}),RangeError);
 assert.throws(()=>emitSystemVerilog({...program}),e=>e.code==='HDL_UNVERIFIED_AST');
 const d=mkdtempSync(join(tmpdir(),'hdl-checked-boundary-'));
 try{
 const rtl=join(d,'guest.sv'),exe=join(d,'sim');
 const tb="module tb; logic signed [63:0] x,y; wire signed [63:0] v; wire f; ores_provider_function0 dut(x,y,v,f); initial begin x=-64'sd10;y=64'sd4;#1;if(f!==0||v!==-64'sd38)$fatal(1,\"negative\");x=64'sh7fffffffffffffff;y=2;#1;if(f!==1||v!==0)$fatal(1,\"overflow\");x=10;y=4;#1;if(f!==0||v!==42)$fatal(1,\"recovery\");$display(\"HDL_CHECKED_BOUNDARY|negative|overflow-fault|recovery\");$finish;end endmodule";
 writeFileSync(rtl,systemVerilog+'\n'+tb);
 const c=spawnSync('iverilog',['-g2012','-s','tb','-o',exe,rtl],{encoding:'utf8',timeout:10000});assert.equal(c.status,0,c.stderr);
 const p=spawnSync('vvp',[exe],{encoding:'utf8',timeout:5000});assert.equal(p.status,0,p.stderr);assert.match(p.stdout,/HDL_CHECKED_BOUNDARY/);console.log('HDL_CHECKED_BOUNDARY|negative|overflow-fault|recovery');
 }finally{rmSync(d,{recursive:true,force:true})}
});
