// Adapted closed-scalar compiler test, not native namespace parser/linker.
// Authenticate all original files before lowering only the selected declarations.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,lstatSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {compile,IncrementalSession} from '../src/compiler.mjs';
const hash=s=>createHash('sha256').update(s).digest('hex');
const pin='8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae';
const edit='85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e';
const abi='dbe6a14b22fba63b297c140b7b6b0f0e49e6a46b5646fc8d23682fd0c66ef069';
const mp='72721b058d5367d74ed1bc4694e79ad13742db24513a192fb25851a3d391912f';
const root=process.env.ATTO_V3_FIXTURE;
function digest(files){return hash([...files].sort(([a],[b])=>a<b?-1:a>b?1:0).map(([p,s])=>p+'\n'+hash(s)+'\n').join(''))}
function adapt(files,expected,ns,ids){
 assert.equal(files.size,12);assert.equal(digest(files),expected,'stale source pin');
 const functions=new Map();
 for(const [path,source] of files){
  const m=/^namespace (billing|identity|telemetry);\npub fnc (function[0-9]+)\(([^)]*)\): int \{ return ([^;]+); \}\n$/.exec(source);
  if(!m){assert.match(path,/^src\/local[01]\.ores$/);assert.match(source,/^pub fnc local[01]\(\): int \{ return (10|11); \}\n$/);continue;}
  assert.equal(path,'src/unit'+m[2].slice(8)+'.ores');
  assert.ok(!functions.has(m[2]));functions.set(m[2],{ns:m[1],decl:source.slice(source.indexOf('\n')+1).replace(/^pub /,'')});
 }
 let declarations='',expr='0';
 for(const i of ids){const name='function'+i,f=functions.get(name);assert.equal(f.ns,ns);declarations+=f.decl;expr='('+expr+')*100+'+name+'()';}
 return declarations+'pub fnc main(): int { return '+expr+'; }';
}
const run=async code=>(await import('data:text/javascript;base64,'+Buffer.from(code).toString('base64'))).main();
test('shared v3 closed scalar ESM differential and fragment reuse',async()=>{
 const files=new Map();
 for(const f of readdirSync(join(root,'checkout/src')).sort()){
  const p=join(root,'checkout/src',f);assert.ok(lstatSync(p).isFile());files.set('src/'+f,readFileSync(p,'utf8'));
 }
 assert.equal(hash(readFileSync(join(root,'checkout/.ores.pkg.toml'))),mp);
 for(const [client,ns,ids] of [['client','billing',[0,1,2,3]],['observer','identity',[4,5,6]]]){
  const manifest=readFileSync(join(root,client,'.ores.pkg.toml'),'utf8');assert.ok(manifest.includes(pin)&&manifest.includes(mp));
  assert.equal(readFileSync(join(root,client,'src/main.ores'),'utf8'),'namespace app;\nimport namespace '+ns+' as provider from "checkout";\npub fnc main(): void {\n'+ids.map(i=>'  stdio.println(provider.function'+i+'());\n').join('')+'  return;\n}\n');
 }
 const billing=(f,p)=>adapt(f,p,'billing',[0,1,2,3]),observer=(f,p)=>adapt(f,p,'identity',[4,5,6]);
 const s=new IncrementalSession(),o=new IncrementalSession();
 const initial=billing(files,pin),other=observer(files,pin);
 const cold=s.build(initial,{cold:true});assert.deepEqual([cold.emitted,cold.reused],[5,0]);assert.equal(cold.code,compile(initial));assert.equal(await run(cold.code),10203n);
 const warm=s.build(initial);assert.deepEqual([warm.emitted,warm.reused],[0,5]);assert.equal(warm.code,cold.code);
 const oc=o.build(other);assert.deepEqual([oc.emitted,oc.reused],[4,0]);assert.equal(await run(oc.code),40506n);
 const changed=new Map(files);changed.set('src/unit0.ores',files.get('src/unit0.ores').replace('return 0','return 42'));
 assert.throws(()=>billing(changed,pin),/stale source pin/);
 const edited=billing(changed,edit),ec=s.build(edited);assert.deepEqual([ec.emitted,ec.reused],[1,4]);assert.equal(ec.code,compile(edited));assert.equal(await run(ec.code),42010203n);
 const oe=o.build(observer(changed,edit));assert.deepEqual([oe.emitted,oe.reused],[0,4]);assert.equal(oe.code,oc.code);assert.equal(await run(oe.code),40506n);
 const bad=new Map(changed);bad.set('src/unit0.ores','namespace billing;\npub fnc function0(int x): int { return x; }\n');
 const rejected=billing(bad,abi);let diagnostic;try{compile(rejected)}catch(e){diagnostic=e.message}assert.ok(diagnostic);assert.throws(()=>s.build(rejected),e=>e.message===diagnostic);
 const recovered=s.build(edited);assert.deepEqual([recovered.emitted,recovered.reused],[0,5]);assert.equal(recovered.code,ec.code);
 if(process.env.ATTO_V3_PARAMETERIZED){
  let previous;
  for(const [add,p,expected] of [[2,'541dfd09d7f18114efd7ec9711721c57a9b14695624a72046425e6db22ccad80',42010203n],[3,'4e1e57d3e7a37978f99dc79399799a994e3d06d7284cbb8e4eb09e46a7c531e7',43010203n]]){
   const f=new Map(files);f.set('src/unit0.ores','namespace billing;\npub fnc function0(int x, int y): int { return x * y + '+add+'; }\n');
   const invalid=billing(f,p),valid=invalid.replace('+function0()','+function0(5+5,4)');assert.notEqual(valid,invalid);
   const built=s.build(valid);assert.equal(built.code,compile(valid));assert.equal(await run(built.code),expected);
   const warm=s.build(valid);assert.deepEqual([warm.emitted,warm.reused],[0,5]);assert.equal(warm.code,built.code);
   let diagnostic;try{compile(invalid)}catch(e){diagnostic=e.message}assert.ok(diagnostic);assert.throws(()=>s.build(invalid),e=>e.message===diagnostic);
   const again=s.build(valid);assert.deepEqual([again.emitted,again.reused],[0,5]);assert.equal(again.code,built.code);
   if(previous!==undefined)assert.notEqual(previous,built.code);previous=built.code;
  }
  console.log('V3_JS_PARAMETERIZED|guest=42010203/43010203|clean-differential|warm|arity-denied|rollback');
 }
 console.log('V3_JS_ADAPTED_ESM|actual=10203/42010203/40506|fragments=5/0,0/5,1/4|observer=0/4|byte-stable|stale-pin|abi-denied|rollback');
});
