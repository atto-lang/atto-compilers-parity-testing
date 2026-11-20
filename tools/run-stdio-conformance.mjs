#!/usr/bin/env node
// Run real source compiler + guest processes for the canonical stdio corpus.
// NEVER use host-side emulation as a replacement for compiled .atto execution.
// Configuration is an explicit, trusted local file; no shell is involved.
import {readFileSync, existsSync, mkdtempSync, rmSync, statSync, realpathSync} from 'node:fs';
import {spawn, spawnSync} from 'node:child_process';
import {resolve, join, sep, isAbsolute} from 'node:path';
import {tmpdir} from 'node:os';
import {createHash} from 'node:crypto';

export const BACKENDS=Object.freeze([
  'java','llvm','forge','net','wasm','rs','js','cranelift','atto','hdl','realtime'
]);
const SHA40=/^[0-9a-f]{40}$/;
const hash=value=>createHash('sha256').update(value).digest('hex');
const error=(message)=>{throw Error(message)};
const assert=(condition,message)=>{if(!condition)error(message)};
const sourcePath=(root,path)=>{
  assert(typeof path==='string'&&path.endsWith('.atto'),'invalid .atto fixture path');
  const base=realpathSync(root);
  const full=realpathSync(resolve(base,path));
  assert(full.startsWith(base+sep),'fixture escaped conformance root');
  return full;
};
export function validateConfig(config) {
  assert(config&&typeof config==='object'&&!Array.isArray(config),'configuration required');
  assert(BACKENDS.includes(config.backend),'unknown backend');
  assert(typeof config.toolchain==='string'&&config.toolchain.trim(),'toolchain description required');
  for(const key of ['compilerRepo','suiteRepo']) {
    const repo=config[key];
    assert(repo&&typeof repo==='object'&&isAbsolute(repo.path)&&SHA40.test(repo.sha),
      key+' must have absolute path and exact 40-character Git head SHA');
  }
  assert(Array.isArray(config.compile)&&config.compile.length>0,'compile argv required');
  assert(config.compile.every(x=>typeof x==='string'&&x),'invalid compile argv');
  if(config.backend!=='hdl') {
    assert(Array.isArray(config.execute)&&config.execute.length>0,'guest execute argv required');
    assert(config.execute.every(x=>typeof x==='string'&&x),'invalid guest argv');
  }
  const hasSource=config.compile.some(x=>x.includes('{source}'));
  const hasArtifact=config.compile.some(x=>x.includes('{artifact}'));
  assert(hasSource&&hasArtifact,'compile argv must reference source and artifact');
  if(config.backend!=='hdl')
    assert(config.execute.some(x=>x.includes('{artifact}')),'guest argv must reference compiled artifact');
  if(config.timeoutMs!==undefined)
    assert(Number.isSafeInteger(config.timeoutMs)&&config.timeoutMs>=1000&&config.timeoutMs<=120000,
      'timeoutMs outside 1-120 seconds');
  return config;
}
function gitHead(root) {
  const r=spawnSync('git',['-C',root,'rev-parse','HEAD'],
    {encoding:'utf8',timeout:8000,maxBuffer:4096});
  if(r.error||r.status!==0)error('cannot inspect git HEAD: '+root);
  const sha=r.stdout.trim().toLowerCase();
  assert(SHA40.test(sha),'invalid checked-out Git HEAD');
  return sha;
}
function argv(template,source,artifact) {
  return template.map(x=>x.replaceAll('{source}',source).replaceAll('{artifact}',artifact));
}
function cleanEnv(extra={}) {
  const keys=['PATH','HOME','TMPDIR','TMP','TEMP','SystemRoot','LANG','LC_ALL'];
  const env=Object.fromEntries(keys.filter(k=>process.env[k]!==undefined)
    .map(k=>[k,process.env[k]]));
  for(const [key,value] of Object.entries(extra)) {
    assert(/^[A-Za-z_][A-Za-z_0-9]*$/.test(key) &&
      typeof value==='string'&&value.length<=4096,'invalid isolated env override');
    assert(!/TOKEN|SECRET|PASSWORD|KEY|CREDENTIAL/i.test(key),
      'secret-like environment keys must not reach the guest process');
    env[key]=value;
  }
  return env;
}
async function run(argv,input,timeoutMs,extraEnv={}) {
  assert(argv.length>0,'empty command line');
  // Deliberately no shell interpolation; no ambient GitHub secrets or FDs.
  const child=spawn(argv[0],argv.slice(1),{
    env:cleanEnv(extraEnv),stdio:['pipe','pipe','pipe'],shell:false,
    windowsHide:true,
  });
  const out=[],err=[];
  let outBytes=0,errBytes=0,overflow=false,timeout=false;
  const cap=2*1024*1024;
  const consume=(chunks,kind)=>(part)=>{
    if(kind==='out')outBytes+=part.length;else errBytes+=part.length;
    if(outBytes>cap||errBytes>cap){overflow=true;child.kill('SIGKILL');return;}
    chunks.push(Buffer.from(part));
  };
  child.stdout.on('data',consume(out,'out'));
  child.stderr.on('data',consume(err,'err'));
  child.stdin.on('error',()=>{}); // early compiler failure may close stdin.
  child.stdin.end(input);
  const timer=setTimeout(()=>{timeout=true;child.kill('SIGKILL')},timeoutMs);
  const result=await new Promise(resolve=>{
    child.once('error',e=>resolve({error:String(e),status:null,signal:null}));
    child.once('close',(status,signal)=>resolve({status,signal}));
  });
  clearTimeout(timer);
  return {...result,timeout,overflow,stdout:Buffer.concat(out),
    stderr:Buffer.concat(err)};
}
const explain=r=>r.error||r.timeout?'process failed or timed out':
  r.overflow?'unbounded subprocess output':'unexpected process outcome';
function resultText(r){return Buffer.concat([r.stdout,Buffer.from('\n'),r.stderr]).toString('utf8')}
function noArtifact(path){return !existsSync(path)||statSync(path).size===0}
export function parseSuiteManifest(s) {
  const c=JSON.parse(s);
  assert(c.schema==='atto.stdio.channel.endpoints.v1','wrong suite contract');
  assert(c.status==='TARGET_NOT_EXECUTED','suite status was unexpectedly promoted');
  assert(c.backend_status&&BACKENDS.every(x=>c.backend_status[x]==='NOT_EXECUTED'),
    'missing backend or forged completion');
  const cases=c.required_cases;
  assert(Array.isArray(cases)&&cases.length>=15,'incomplete stdio case coverage');
  const ids=new Set(),byId=new Map();
  for(const item of cases) {
    assert(typeof item.id==='string'&&!ids.has(item.id),'duplicate suite case');
    ids.add(item.id);byId.set(item.id,item);
    assert(['guest-pass','compile-fail','source-compile-pass','compile-or-runtime-deny'].includes(item.kind),
      'unsupported case type');
    if(item.kind==='compile-fail')
      assert(/^E_[A-Z_]+$/.test(item.expected_error)&&item.no_guest_artifact===true,
        'compile-fail must require exact diagnostic and zero artifact');
  }
  const guests=['checked_echo','checked_echo_readch_alias',
    'checked_echo_is_variants','checked_echo_is_variants_readch'];
  const negatives=['unchecked_read_payload','wrong_read_direction',
    'wrong_write_direction','read_without_suspend','stream_and_readch_conflict',
    'is_unknown_extractor','is_none_payload','is_void_ok_bound',
    'is_missing_ok_payload','is_wrong_variant_family','is_nested_extractor',
    'is_multi_payload','is_duplicate_arm'];
  for(const id of guests)assert(byId.get(id)?.kind==='guest-pass','missing checked guest '+id);
  for(const id of negatives) {
    const kind=id==='stream_and_readch_conflict'?'compile-or-runtime-deny':'compile-fail';
    assert(byId.get(id)?.kind===kind,'missing or weakened negative '+id);
  }
  assert(byId.get('stdio_is_pattern_fallback')?.kind==='source-compile-pass',
    'missing source-only nested string fallback');
  assert(cases.length===guests.length+negatives.length+1,'extra or missing stdio scenarios');
  const baseline=JSON.stringify(byId.get('checked_echo').runs);
  for(const id of guests) {
    const item=byId.get(id);
    assert(JSON.stringify(item.runs)===baseline,'stdio echo aliases must have identical byte witnesses');
    assert(Array.isArray(item.runs)&&item.runs.length===7,'all seven exact stdin scenarios required');
    for(const witness of item.runs)
      assert(typeof witness.stdin==='string'&&typeof witness.stdout==='string'&&
        typeof witness.stderr==='string'&&Number.isSafeInteger(witness.exit),
        'incomplete per-guest byte witness');
  }
  return c;
}
function checkExit(result,kind,expected) {
  if(result.error||result.timeout||result.overflow)
    return explain(result)+': '+String(result.error||'');
  if(kind==='success')return result.status===0?null:'exit '+result.status;
  if(kind==='failure') {
    if(result.status===0)return 'compiler accepted invalid guest';
    if(!resultText(result).includes(expected))return 'missing diagnostic '+expected;
    return null;
  }
  return 'unexpected check kind';
}
export async function runSuite(config) {
  validateConfig(config);
  for(const key of ['compilerRepo','suiteRepo']) {
    const expected=config[key].sha,actual=gitHead(config[key].path);
    assert(actual===expected,key+' HEAD changed: required '+expected+' but observed '+actual);
  }
  const suiteRoot=realpathSync(config.suiteRepo.path);
  const suiteFile=readFileSync(join(suiteRoot,'contracts/stdio-channel-endpoints-v1.json'));
  const suite=parseSuiteManifest(suiteFile.toString('utf8'));
  const hasher=createHash('sha256');hasher.update(suiteFile);
  for(const path of suite.required_cases.map(x=>x.path).sort()){
    hasher.update(path);hasher.update(Buffer.from([0]));
    hasher.update(readFileSync(sourcePath(suiteRoot,path)));
  }
  const temp=mkdtempSync(join(tmpdir(),'atto-stdio-guest-'));
  const checks=[];
  try{
    for(const test of suite.required_cases) {
      const source=sourcePath(suiteRoot,test.path);
      const artifact=join(temp,checks.length+'-guest');
      const executeMs=config.timeoutMs??15000;
      const compiled=await run(argv(config.compile,source,artifact),Buffer.alloc(0),executeMs,config.env);
      let reason=null;
      if(config.backend==='hdl') {
        // Hardware targets have no ambient process channels. Compiler must
        // refuse *every* stdio test with a concrete unsupported-profile error.
        reason=checkExit(compiled,'failure','E_UNSUPPORTED_PROFILE');
        if(!noArtifact(artifact))reason='HDL emitted artifact despite unsupported stdio';
        checks.push({id:test.id,status:reason?'FAILED':'UNSUPPORTED_PROFILE_REJECTED',reason});
        continue;
      }
      if(test.kind==='compile-fail') {
        reason=checkExit(compiled,'failure',test.expected_error);
        if(!noArtifact(artifact))reason='compiler emitted artifact for invalid fixture';
        checks.push({id:test.id,status:reason?'FAILED':'COMPILE_REJECTED',reason});
        continue;
      }
      if(test.kind==='compile-or-runtime-deny') {
        // Accept only compile-time denial until an independently checked
        // runtime lease race harness proves no bytes were consumed.
        reason=checkExit(compiled,'failure',test.expected_error);
        if(!noArtifact(artifact))reason='unsafe stream conflict generated artifact';
        checks.push({id:test.id,status:reason?'FAILED':'COMPILE_REJECTED',reason});
        continue;
      }
      reason=checkExit(compiled,'success');
      if(!reason&&(noArtifact(artifact)||statSync(artifact).size>64*1024*1024))
        reason='missing, empty or oversized guest artifact';
      if(reason) {
        checks.push({id:test.id,status:'FAILED',reason});
        continue;
      }
      const artifactSha256=hash(readFileSync(artifact));
      if(test.kind==='source-compile-pass') {
        checks.push({id:test.id,status:'COMPILED_NOT_EXECUTED',artifactSha256});
        continue;
      }
      const cases=[];
      for(const scenario of test.runs) {
        const guest=await run(argv(config.execute,source,artifact),
          Buffer.from(scenario.stdin,'utf8'),executeMs,config.env);
        const failure=guest.error||guest.timeout||guest.overflow?
          explain(guest):
          guest.status!==scenario.exit?'unexpected exit '+guest.status:
          !guest.stdout.equals(Buffer.from(scenario.stdout,'utf8'))?'stdout mismatch':
          !guest.stderr.equals(Buffer.from(scenario.stderr,'utf8'))?'stderr mismatch':null;
        cases.push({id:scenario.id,status:failure?'FAILED':'GUEST_EXECUTED',reason:failure,
          exit:guest.status,stdoutSha256:hash(guest.stdout),stderrSha256:hash(guest.stderr)});
      }
      checks.push({id:test.id,status:cases.some(x=>x.status==='FAILED')?'FAILED':'GUEST_EXECUTED',
        artifactSha256,cases});
    }
  } finally {rmSync(temp,{recursive:true,force:true})}
  const failures=checks.filter(x=>x.status==='FAILED');
  const requiredGuests=checks.filter(x=>x.status==='GUEST_EXECUTED');
  const expectedGuests=suite.required_cases.filter(x=>x.kind==='guest-pass').length;
  const complete=failures.length===0&&
    (config.backend==='hdl'||requiredGuests.length===expectedGuests);
  return {schema:'atto.stdio.local.runner.observation.v1',backend:config.backend,
    sourceHead:config.compilerRepo.sha,suiteHead:config.suiteRepo.sha,
    caseSetSha256:hasher.digest('hex'),toolchain:config.toolchain,
    compiledGuestExecution:config.backend!=='hdl'&&requiredGuests.length===expectedGuests,
    checks,complete,failures:failures.length,
    // A local observation is NOT proof of repository CI, provenance,
    // independent review or readiness; do not convert to release PASS.
    releaseGate:'NOT_VERIFIED'};
}
if(process.argv[1]&&resolve(process.argv[1])===resolve(new URL(import.meta.url).pathname)) {
  if(process.argv.length!==3) {
    console.error('usage: node tools/run-stdio-conformance.mjs /absolute/path/to/config.json');
    process.exitCode=2;
  } else {
    try {
      const config=JSON.parse(readFileSync(resolve(process.argv[2]),'utf8'));
      const report=await runSuite(config);
      process.stdout.write(JSON.stringify(report,null,2)+'\n');
      if(!report.complete)process.exitCode=1;
    } catch(e) {console.error(String(e.stack||e));process.exitCode=2}
  }
}
