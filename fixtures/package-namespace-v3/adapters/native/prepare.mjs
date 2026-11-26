// Test-only adaptation: independent scalar units, not native namespace linkage.
// Exact input pins, no generated output oracle substituted for compiler execution.
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,lstatSync,mkdirSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
const [root,out]=process.argv.slice(2);assert.ok(root&&out);
const hash=x=>createHash('sha256').update(x).digest('hex');
const pin='8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae',edit='85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e',abi='dbe6a14b22fba63b297c140b7b6b0f0e49e6a46b5646fc8d23682fd0c66ef069';
const mp='72721b058d5367d74ed1bc4694e79ad13742db24513a192fb25851a3d391912f';
const files=new Map(readdirSync(join(root,'checkout/src')).sort().map(f=>{const p=join(root,'checkout/src',f);assert.ok(lstatSync(p).isFile());return ['src/'+f,readFileSync(p,'utf8')]}));
function digest(f){return hash([...f].sort(([a],[b])=>a<b?-1:a>b?1:0).map(([p,s])=>p+'\n'+hash(s)+'\n').join(''))}
assert.equal(files.size,12);assert.equal(hash(readFileSync(join(root,'checkout/.ores.pkg.toml'))),mp);assert.equal(digest(files),pin);
for(const [client,ns,ids] of [['client','billing',[0,1,2,3]],['observer','identity',[4,5,6]]]){
 const m=readFileSync(join(root,client,'.ores.pkg.toml'),'utf8');assert.ok(m.includes(pin)&&m.includes(mp));
 const expected='namespace app;\nimport namespace '+ns+' as provider from "checkout";\npub fnc main(): void {\n'+ids.map(i=>'  stdio.println(provider.function'+i+'());\n').join('')+'  return;\n}\n';
 assert.equal(readFileSync(join(root,client,'src/main.ores'),'utf8'),expected);
}
function emit(f,p,lane){
 assert.equal(digest(f),p,'stale source pin');mkdirSync(join(out,lane),{recursive:true});
 for(let i=0;i<10;i++){
  const ns=i<4?'billing':i<7?'identity':'telemetry',s=f.get('src/unit'+i+'.ores');
  const m=new RegExp('^namespace '+ns+';\\npub fnc function'+i+'\\(([^)]*)\\): int \\{ return ([^;]+); \\}\\n$').exec(s);assert.ok(m);
  if(i>=7)continue;
  if(m[1]!==''){assert.equal(lane,'abi');assert.equal(i,0); // Keep incompatible declaration as actual compiler input for LLVM/Forge.
   writeFileSync(join(out,lane,'call'+i+'.atto'),s.slice(s.indexOf('\n')+1)+'pub fnc main(): int { return function'+i+'(); }\n');
   continue;
  }
  // Cranelift's one-entry grammar has no helper calls. Rename only the
  // validated provider declaration; do not interpret its return expression.
  writeFileSync(join(out,lane,'unit'+i+'.atto'),s.slice(s.indexOf('\n')+1).replace('function'+i+'(','main('));
  // Real calls for LLVM and Forge, with the original function body intact.
  writeFileSync(join(out,lane,'call'+i+'.atto'),s.slice(s.indexOf('\n')+1)+'pub fnc main(): int { return function'+i+'(); }\n');
 }
 for(let i=0;i<2;i++)assert.equal(f.get('src/local'+i+'.ores'),'pub fnc local'+i+'(): int { return '+(10+i)+'; }\n');
}
emit(files,pin,'cold');const changed=new Map(files);changed.set('src/unit0.ores',files.get('src/unit0.ores').replace('return 0','return 42'));
assert.throws(()=>emit(changed,pin,'stale'),/stale source pin/);emit(changed,edit,'edit');
const bad=new Map(changed);bad.set('src/unit0.ores','namespace billing;\npub fnc function0(int x): int { return x; }\n');emit(bad,abi,'abi');
console.log('V3_NATIVE_ADAPTER|12-authenticated|7-selected|stale-pin-denied|test-only-lowering');
