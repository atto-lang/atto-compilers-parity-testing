import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync,mkdtempSync,rmSync,existsSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {tmpdir} from 'node:os';
import {pathToFileURL} from 'node:url';

const SITE_HEAD='b8b22a955fa8d4f4018734f1ecb5758773a00f64';
const SITE_TREE='359accbbf1564de2dd359b93d8b2a6439d8c1211';
const [lane,siteArg,compilerArg]=process.argv.slice(2);
const id=lane==='java-aot'?'java':lane==='net-aot'?'net':lane;
assert.ok(['rs','js','llvm','forge','cranelift','java','java-aot','net','net-aot'].includes(lane),'explicit supported probe profile required');
const site=resolve(siteArg), root=resolve(compilerArg);
const sha256=b=>createHash('sha256').update(b).digest('hex');
const blob=b=>createHash('sha1').update('blob '+b.length+'\0').update(b).digest('hex');
function invoke(cmd,args,cwd){
  const r=spawnSync(cmd,args,{cwd,encoding:'utf8',timeout:180000,maxBuffer:1048576,env:{PATH:process.env.PATH,HOME:process.env.HOME,LANG:'C.UTF-8',LC_ALL:'C.UTF-8',...(process.env.JAVA_HOME?{JAVA_HOME:process.env.JAVA_HOME}:{}),...(process.env.DOTNET_ROOT?{DOTNET_ROOT:process.env.DOTNET_ROOT}:{}),DOTNET_CLI_TELEMETRY_OPTOUT:'1',DOTNET_NOLOGO:'1'}});
  return {status:r.status,signal:r.signal,error:r.error?.message??null,stdout:r.stdout??'',stderr:r.stderr??''};
}
function git(dir,...args){const x=invoke('git',args,dir);assert.equal(x.status,0,x.stderr);assert.equal(x.error,null);return x.stdout.trim();}
function pin(dir,commit,tree){assert.equal(git(dir,'rev-parse','HEAD'),commit);assert.equal(git(dir,'rev-parse','HEAD^{tree}'),tree);assert.equal(git(dir,'status','--porcelain','--untracked-files=no'),'');}
pin(site,SITE_HEAD,SITE_TREE);
const m=JSON.parse(readFileSync(join(site,'comparisons/parity-candidates.json'),'utf8'));
assert.equal(m.schema,'atto.comparator.parity-candidates.v1');
const b=m.backends.find(x=>x.id===id);assert.ok(b);
pin(root,b.commit,b.tree);
assert.equal(m.cases.length,33);
const cases=m.cases.map(c=>{
  assert.match(c.name,/^[a-z0-9-]+$/);
  assert.equal(c.path,'play/examples/preview/'+c.name+'.atto');
  const src=join(site,c.path);assert.equal(blob(readFileSync(src)),c.blob);
  assert.match(c.return,/^(0|[1-9][0-9]*)$/);
  assert.ok(typeof c.stdout==='string');
  return {...c,src};
});
assert.equal(new Set(cases.map(c=>c.name)).size,cases.length);
const tmp=mkdtempSync(join(tmpdir(),'atto-comparator-'));
const result=[];
const negatives=[];
const javaJar=join(root,'target/oreslang-source-0.1.0-hotload-SNAPSHOT.jar');
const javaNative=join(root,'target/native/liboresthread.so');
const javaArgs=id==='java'&&lane==='java'?['--enable-native-access=ALL-UNNAMED','-Dpolyglot.engine.WarnInterpreterOnly=false','-Dores.thread.native.path='+javaNative,'-cp',javaJar+':'+readFileSync(join(root,'target/conformance-classpath.txt'),'utf8').trim(),'dev.oreslang.launcher.OresMain','--mode=aot','--no-core-log']:[];
const javaCmd=lane==='java-aot'?join(root,'target/ores-aot'):'java';
const netCompiler=join(root,'compiler/Atto.Cil.Compiler/Atto.Cil.Compiler.csproj');
const netHost=join(root,'tests/Atto.Cil.Guest.Host/Atto.Cil.Guest.Host.csproj');
const netGuest=join(root,'artifacts/AttoGuest.dll');
if(id==='net'){assert.ok(existsSync(join(root,'artifacts')));assert.ok(!existsSync(netGuest),'stale guest artifact');}
function validProcess(r){return !r.error&&!r.signal&&Number.isInteger(r.status);}
function exitObservation(c,r,artifact,extra={}){if(!validProcess(r)||r.stderr!==''||BigInt(c.return)>255n)return {observation:'GUEST_EXECUTION_UNVERIFIED',status:r.status,diagnostic:r.stderr.slice(0,500)};return status(c,String(r.status),r.stdout,artifact,{return_encoding:'process_exit_status_0_255_only',...extra});}
const errorKind=r=>r.error||r.signal?'PROCESS_ERROR':'COMPILE_OR_RUN_REJECTED';
const compileFailure=(r,artifact)=>({observation:existsSync(artifact)?'REJECTED_WITH_ARTIFACT':errorKind(r),diagnostic:r.stderr.slice(0,500),status:r.status});
const status=(c,value,stdout,artifact,extra={})=>({observation:value===c.return&&stdout===c.stdout?'MATCHED_POSITIVE_ORACLE':'ORACLE_MISMATCH',value,stdout,artifact_sha256:artifact&&existsSync(artifact)?sha256(readFileSync(artifact)):null,...extra});
const jsRunner=join(tmp,'js-runner.mjs');
writeFileSync(jsRunner,`import {pathToFileURL} from 'node:url';const m=await import(pathToFileURL(process.argv[2]));const v=await m.main();if(typeof v!=='bigint'&&!(typeof v==='number'&&Number.isSafeInteger(v)))throw Error('not an integer main result');process.stderr.write('ATTO_PROBE_RETURN='+String(v)+'\\n');`);
const wasmRunner=join(tmp,'wasm-runner.mjs');
writeFileSync(wasmRunner,`import {readFileSync} from 'node:fs';import {pathToFileURL} from 'node:url';import {join} from 'node:path';const base=process.argv[3];const {guestImports,usesStateLeases}=await import(pathToFileURL(join(base,'play/src/guest.js')));const {createStateRuntime}=await import(pathToFileURL(join(base,'play/src/state-runtime.js')));const m=await WebAssembly.compile(readFileSync(process.argv[2]));const out={text:'',truncated:false};const state=usesStateLeases(m)?createStateRuntime():null;const {imports,bind}=guestImports(m,out,null,null,{state});const i=await WebAssembly.instantiate(m,imports);bind(i);const v=i.exports.main();state?.complete();if(out.truncated||typeof v!=='bigint'||i.exports.panic_code?.value!==0)throw Error('invalid guest outcome');process.stdout.write(out.text);process.stderr.write('ATTO_PROBE_RETURN='+String(v)+'\\n');`);
const cWrapper=join(tmp,'entry.c');
writeFileSync(cWrapper,'#include <stdint.h>\n#include <stdio.h>\n#include <inttypes.h>\nextern int64_t oresforge_entry(void);\nint main(void){fprintf(stderr,"ATTO_PROBE_RETURN=%" PRId64 "\\n",oresforge_entry());return 0;}\n');
function tagged(c,r,artifact){const m=r.stderr.match(/^ATTO_PROBE_RETURN=(-?\d+)\n$/);if(r.status!==0||r.error||r.signal||!m)return {observation:'GUEST_EXECUTION_UNVERIFIED',status:r.status,diagnostic:r.stderr.slice(0,500)};return status(c,m[1],r.stdout,artifact);}
try{
  for(const c of cases){
    const out=join(tmp,c.name+(id==='rs'?'.wasm':id==='js'?'.mjs':'.bin'));
    let o;
    if(id==='java'){
      const cc=invoke(javaCmd,[...javaArgs,'--check',c.src],root);
      if(cc.status!==0||!validProcess(cc))o=compileFailure(cc,out);
      else o=exitObservation(c,invoke(javaCmd,[...javaArgs,c.src],root),null,{execution:lane==='java-aot'?'native_image_host':'jvm_host_guest_jit_disabled'});
    }else if(id==='net'){
      rmSync(netGuest,{force:true});
      const cc=invoke('dotnet',['run','--no-build','-c','Release','--project',netCompiler,'--',c.src,netGuest,'--allow-stdio'],root);
      if(cc.status!==0||!validProcess(cc)||!existsSync(netGuest))o=compileFailure(cc,netGuest);
      else{
        const cilSha=sha256(readFileSync(netGuest));
        const dir=join(tmp,c.name+'-host');
        const buildArgs=lane==='net-aot'?['publish',netHost,'-c','Release','-r','linux-x64','-p:PublishAot=true','--self-contained','true','-o',dir]:['build',netHost,'-c','Release','--no-incremental','-o',dir];
        const build=invoke('dotnet',buildArgs,root);
        const host=join(dir,lane==='net-aot'?'Atto.Cil.Guest.Host':'Atto.Cil.Guest.Host.dll');
        if(build.status!==0||!validProcess(build)||!existsSync(host))o={observation:'HOST_BUILD_FAILED',status:build.status,diagnostic:(build.stderr+' '+build.stdout).slice(-1200),cil_sha256:cilSha};
        else{
          const run=lane==='net-aot'?invoke(host,[],tmp):invoke('dotnet',[host],tmp);
          assert.equal(sha256(readFileSync(netGuest)),cilSha);
          o=exitObservation(c,run,host,{cil_sha256:cilSha,execution:lane==='net-aot'?'native_aot':'coreclr'});
        }
      }
      rmSync(netGuest,{force:true});
    }else if(id==='rs'){
      const cc=invoke(join(root,'target/release/attoc'),[c.src,'-o',out],root);
      o=cc.status===0&&!cc.error&&!cc.signal&&existsSync(out)?tagged(c,invoke(process.execPath,[wasmRunner,out,site],tmp),out):compileFailure(cc,out);
    }else if(id==='js'){
      const pipeline=c.name==='function-pipeline';
      const cli=join(root,pipeline?'bin/atto-jsc.mjs':'bin/oreslang-jsc.mjs');
      const cc=invoke(process.execPath,[cli,pipeline?'--functional-source':'--atto-scalar-v1',c.src,'--out='+out],root);
      o=cc.status===0&&!cc.error&&!cc.signal&&existsSync(out)?tagged(c,invoke(process.execPath,[jsRunner,out],tmp),out):compileFailure(cc,out);
    }else if(id==='llvm'){
      const cc=invoke('bash',[join(root,'scripts/compile-aot.sh'),c.src,out],root);
      if(cc.status!==0||cc.error||cc.signal||!existsSync(out))o=compileFailure(cc,out);
      else {const rr=invoke(out,[],tmp);o=rr.error||rr.signal||rr.status===null||rr.stderr!==''?{observation:'GUEST_EXECUTION_UNVERIFIED',status:rr.status,diagnostic:rr.stderr.slice(0,500)}:status(c,String(rr.status),rr.stdout,out,{return_encoding:'process_exit_status_0_255_only'});}
    }else if(id==='forge'){
      const asm=out+'.s';const cc=invoke(join(root,'build/oresforge'),['emit-runtime-asm','x86_64',c.src],root);
      if(cc.status!==0||cc.error||cc.signal||!cc.stdout.includes('oresforge_entry:'))o=compileFailure(cc,out);
      else{writeFileSync(asm,cc.stdout);const link=invoke('cc',[cWrapper,asm,'-o',out],tmp);o=link.status===0&&existsSync(out)?tagged(c,invoke(out,[],tmp),out):compileFailure(link,out);}
    }else{
      // The JIT CLI prints its result as a final line. For non-empty expected
      // guest stdout the CLI's combined stream is not an independent oracle.
      const rr=invoke(join(root,'target/release/orescranelift'),['run',c.src],root);
      if(rr.status!==0||rr.error||rr.signal)o={observation:errorKind(rr),diagnostic:rr.stderr.slice(0,500),status:rr.status};
      else if(c.stdout!==''||rr.stderr!=='')o={observation:'GUEST_EXECUTION_UNVERIFIED',diagnostic:'combined JIT stream has no independent stdout receipt'};
      else o=rr.stdout===c.return+'\n'?{observation:'MATCHED_POSITIVE_ORACLE',value:c.return,stdout:'',artifact_sha256:null,execution:'native_jit_cli_return_only'}:{observation:'ORACLE_MISMATCH',stdout:rr.stdout};
    }
    result.push({name:c.name,source_blob:c.blob,...o});
    console.log('COMPARATOR_CASE '+JSON.stringify(result.at(-1)));
  }
  // Pin a known invalid source independently of the positive corpus. A generic
  // compiler failure is not a typed negative match. No generated output may survive.
  const invalid=join(site,'play/examples/error.atto');
  assert.equal(blob(readFileSync(invalid)),'7c7f1673024e607cb4d0599048c346324854d2e5');
  if(id==='rs'){
    const artifact=join(tmp,'immutable-reject.wasm');
    const rr=invoke(join(root,'target/release/attoc'),[invalid,'-o',artifact],root);
    negatives.push({name:'immutable-reassignment',source_blob:blob(readFileSync(invalid)),observation:validProcess(rr)&&rr.status===1&&rr.stderr.startsWith(invalid+': 4:3: E_IMMUTABLE:')&&!existsSync(artifact)?'MATCHED_NEGATIVE_ORACLE':'NEGATIVE_UNVERIFIED',diagnostic:rr.stderr.slice(0,500),artifact_exists:existsSync(artifact)});
    const overflow=join(site,'play/examples/overflow.atto');
    assert.equal(blob(readFileSync(overflow)),'e2ec3bb25ed02e47645946c1db221eed05579ee3');
    const artifact2=join(tmp,'overflow.wasm');
    const cc=invoke(join(root,'target/release/attoc'),[overflow,'-o',artifact2],root);
    const runner=join(tmp,'panic-runner.mjs');
    writeFileSync(runner,`import {readFileSync} from 'node:fs';import {pathToFileURL} from 'node:url';import {join} from 'node:path';const {guestImports,describePanic}=await import(pathToFileURL(join(process.argv[3],'play/src/guest.js')));const mod=await WebAssembly.compile(readFileSync(process.argv[2]));const output={text:'',truncated:false};const {imports,bind}=guestImports(mod,output);const ins=await WebAssembly.instantiate(mod,imports);bind(ins);try{ins.exports.main();process.exitCode=3;}catch(e){if(ins.exports.panic_code.value!==1||describePanic(ins,e)!=='E_PANIC: integer overflow'||output.truncated||!output.text.endsWith('4611686018427387904\\n'))process.exitCode=4;else console.log('E_PANIC: integer overflow');}`);
    const rr2=cc.status===0&&validProcess(cc)&&existsSync(artifact2)?invoke(process.execPath,[runner,artifact2,site],tmp):null;
    negatives.push({name:'checked-overflow',source_blob:blob(readFileSync(overflow)),observation:rr2&&validProcess(rr2)&&rr2.status===0&&rr2.stdout==='E_PANIC: integer overflow\n'&&rr2.stderr===''?'MATCHED_NEGATIVE_ORACLE':'NEGATIVE_UNVERIFIED',artifact_sha256:existsSync(artifact2)?sha256(readFileSync(artifact2)):null,diagnostic:rr2?.stderr??cc.stderr});
  }
  for(const n of negatives)console.log('COMPARATOR_NEGATIVE '+JSON.stringify(n));
  pin(site,SITE_HEAD,SITE_TREE);pin(root,b.commit,b.tree);
  const summary={};for(const v of result)summary[v.observation]=(summary[v.observation]||0)+1;
  console.log('COMPARATOR_PROBE '+JSON.stringify({schema:'atto.comparator.positive-probe.v1',backend:id,lane,compiler_commit:b.commit,compiler_tree:b.tree,site_commit:SITE_HEAD,site_tree:SITE_TREE,node:process.version,summary,negative_coverage:negatives.length?negatives:'NOT_EXECUTED_BY_THIS_PROBE',certified_parity:false}));
  // A failed or rejected positive is a gap, never a green parity job.
  if(result.some(x=>x.observation!=='MATCHED_POSITIVE_ORACLE')||negatives.some(n=>n.observation!=='MATCHED_NEGATIVE_ORACLE'))process.exitCode=1;
}finally{rmSync(tmp,{recursive:true,force:true});}
