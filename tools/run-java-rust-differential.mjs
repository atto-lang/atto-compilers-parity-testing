#!/usr/bin/env node
// Opt-in AttoLang Java/Rust candidate observations; NEVER a guest attestation.
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync, mkdtempSync, lstatSync, realpathSync, existsSync, rmSync, readdirSync} from 'node:fs';
import {canonicalI64, classifyCompileNegative, classifyUnsupported, classifyUndecidedCompile, classifyGuest, assertObservationReport} from './java-rust-observation-policy.mjs';
import {validateWasmAdapterShape, supportsWasmOracle} from './guest/validate-wasm-adapter.mjs';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {resolve, join, relative, isAbsolute} from 'node:path';
import {fileURLToPath} from 'node:url';

const ROOT=resolve(fileURLToPath(new URL('..',import.meta.url)));
const PATH=join(ROOT,'contracts/java-rust-conflicts-v1/manifest.json');
const BACKENDS=['java-graal','rust-wasm','rust-cranelift'];
const hex40=/^[a-f0-9]{40}$/;
const hex64=/^[a-f0-9]{64}$/;
const hash=b=>createHash('sha256').update(b).digest('hex');
function fail(message){throw new Error(message);}
function regular(path){
  if(!isAbsolute(path)||lstatSync(path).isSymbolicLink()||!lstatSync(path).isFile()||realpathSync(path)!==path)fail('untrusted file: '+path);
  return path;
}
function sourcePath(p){
  if(typeof p!=='string'||!/^contracts\/java-rust-conflicts-v1\/fixtures\/[a-z0-9-]+\.atto$/.test(p))fail('invalid source name');
  const s=resolve(ROOT,p);
  if(relative(ROOT,s).startsWith('..'))fail('escaped source root');
  return regular(s);
}
export function loadManifest(){
  const m=JSON.parse(readFileSync(PATH,'utf8'));
  if(m.schema!=='atto.java_rust.differential.v1'||m.status!=='CANDIDATE_NOT_EXECUTED')fail('bad manifest schema');
  if(JSON.stringify(m.scope?.required_backends)!==JSON.stringify(BACKENDS)||m.scope?.baseline!=='NOT_EXECUTED'||m.scope?.no_legacy_53_case_renumbering!==true)fail('wrong backend scope');
  if(m.source_policy?.source_rewrite!=='forbidden'||m.source_policy?.simulated_compilers!=='not_evidence')fail('unsafe source policy');
  if(!Array.isArray(m.cases)||m.cases.length!==10||!Array.isArray(m.baseline)||m.baseline.length!==30)fail('wrong corpus size');
  const seen=new Set();
  for(const c of m.cases){
    if(!/^[a-z][a-z0-9-]+$/.test(c.id)||seen.has(c.id)||c.status!=='AUTHORED_NOT_EXECUTED')fail('duplicate/claimed fixture');
    if(c.path!==`contracts/java-rust-conflicts-v1/fixtures/${c.id}.atto`)fail('fixture identity/path mismatch');
    seen.add(c.id);
    if(!['canonical','candidate','decision_required'].includes(c.authority)||!['value','stdout','reject','trap','observe'].includes(c.kind))fail('bad oracle');
    if((c.kind==='observe')!==(c.authority==='decision_required'))fail('unreviewed oracle');
    if(!hex64.test(c.sha256)||hash(readFileSync(sourcePath(c.path)))!==c.sha256)fail('fixture hash mismatch: '+c.id);
    if(c.kind==='value'&&canonicalI64(c.expected_value)===null)fail('missing/invalid i64 oracle');
    if(['reject','trap'].includes(c.kind)&&!/^[A-Z][A-Z0-9_]+$/.test(c.expected_diagnostic||''))fail('missing targeted diagnostic');
    if(c.kind==='stdout'&&(typeof c.expected_stdout!=='string'||!Number.isInteger(c.expected_exit)))fail('missing stdout oracle');
  }
  const rows=new Set();
  for(const r of m.baseline){
    const key=r.case_id+'/'+r.backend;
    if(!seen.has(r.case_id)||!BACKENDS.includes(r.backend)||rows.has(key)||r.status!=='NOT_EXECUTED'||r.guest_executed!==false||r.executed_steps!==0||r.provenance!==null)fail('forged baseline cell');
    rows.add(key);
  }
  return m;
}
export function audit(){
  const m=loadManifest();
  return {schema:'atto.java_rust.differential.report.v1',mode:'METADATA_ONLY',cases:m.cases.length,records:m.baseline,guest_parity_certified:false};
}
export function parseArgs(args){
  const out={execute:false,backend:null,adapter:null,output:null};
  const seen=new Set();
  for(let i=0;i<args.length;i++){
    const option=args[i];
    if(seen.has(option))fail('duplicate option: '+option);
    seen.add(option);
    if(option==='--audit')continue;
    if(option==='--execute'){out.execute=true;continue;}
    if(!['--backend','--adapter','--output'].includes(option)||!args[i+1]||args[i+1].startsWith('--'))fail('unknown/missing option: '+option);
    out[option.slice(2)]=args[++i];
  }
  if(out.execute&&seen.has('--audit'))fail('--audit cannot also execute');
  if(out.execute&&(!BACKENDS.includes(out.backend)||!out.adapter))fail('execute requires backend and adapter');
  if(!out.execute&&(out.backend||out.adapter))fail('adapter and backend require --execute');
  return out;
}
function checkCommand(argv, executable, needsSource){
  if(!Array.isArray(argv)||argv.length<2||argv.length>64||argv[0]!==executable||argv.some(x=>typeof x!=='string'||!x||x.length>8192||/[\u0000-\u001f]/.test(x)))fail('untrusted argv');
  if(needsSource&&!argv.some(x=>x.includes('{source}')))fail('compile must use source placeholder');
  if(!argv.some(x=>x.includes('{artifact}')))fail('must use artifact placeholder');
  for(const arg of argv)for(const match of arg.matchAll(/\{[^}]*\}/g))if(!['{source}','{artifact}','{workdir}'].includes(match[0]))fail('unsupported placeholder');
}
function checkAdapter(a,b,cases){
  if(a.backend!==b||!hex40.test(a.compiler_commit)||!hex40.test(a.compiler_tree))fail('unpinned compiler commit/tree');
  if(!hex64.test(a.compiler_sha256)||hash(readFileSync(regular(a.compiler_executable)))!==a.compiler_sha256)fail('compiler binary hash mismatch');
  if(!hex64.test(a.guest_runner_sha256)||hash(readFileSync(regular(a.guest_runner_executable)))!==a.guest_runner_sha256)fail('guest runner hash mismatch');
  checkCommand(a.compile_argv,a.compiler_executable,true);
  checkCommand(a.run_argv,a.guest_runner_executable,false);
  if(!['exit_code','stdout_i64'].includes(a.value_mode))fail('unapproved value extraction');
  const names=new Set(cases.map(c=>c.id));
  if(a.unsupported_cases!==undefined&&(!Array.isArray(a.unsupported_cases)||new Set(a.unsupported_cases).size!==a.unsupported_cases.length||a.unsupported_cases.some(x=>!names.has(x))))fail('bad unsupported list');
  if(a.unsupported_cases?.length){
    if(!a.unsupported_needles||typeof a.unsupported_needles!=='object'||Array.isArray(a.unsupported_needles))fail('missing unsupported diagnostic map');
    for(const id of a.unsupported_cases)if(typeof a.unsupported_needles[id]!=='string'||!/^E_UNSUPPORTED(?:_[A-Z0-9_]+)?$/.test(a.unsupported_needles[id]))fail('bad unsupported diagnostic');
  }
  if(a.diagnostic_needles!==undefined)fail('free-form diagnostic needles are unsafe; use fixture category codes');
  if(b==='rust-wasm'){
    const w=validateWasmAdapterShape(a);
    if(hash(readFileSync(regular(w.guest_runner_script)))!==w.guest_runner_script_sha256)
      fail('wasm guest host script hash mismatch');
  }
  return a;
}
function invoke(argv,c){
  const args=argv.map(s=>s.replaceAll('{source}',c.source).replaceAll('{artifact}',c.artifact).replaceAll('{workdir}',c.workdir));
  const p=spawnSync(args[0],args.slice(1),{cwd:c.workdir,shell:false,encoding:'utf8',timeout:15000,maxBuffer:1024*1024,windowsHide:true});
  return {exit_code:p.status,signal:p.signal,stdout:(p.stdout||'').slice(0,4096),stderr:(p.stderr||'').slice(0,4096),error:p.error?.code||null};
}
function runCase(t,a){
  const tmp=mkdtempSync(join(tmpdir(),'atto-diff-'));
  const c={source:sourcePath(t.path),artifact:join(tmp,'guest-artifact'),workdir:tmp};
  try{
    const compile=invoke(a.compile_argv,c);
    if(compile.error||compile.signal)return {status:'INFRASTRUCTURE_FAILURE',compile};
    if(t.kind==='reject'||(a.unsupported_cases||[]).includes(t.id)){
      const absent=readdirSync(c.workdir).length===0;
      const unsupported=(a.unsupported_cases||[]).includes(t.id);
      const status=unsupported
        ?classifyUnsupported(compile,absent,a.unsupported_needles[t.id])
        :classifyCompileNegative(compile,absent,t.expected_diagnostic);
      return {status,compile,artifact_absent:absent};
    }
    if(a.backend==='rust-wasm' && !supportsWasmOracle(t.kind))
      return {status:'INFRASTRUCTURE_FAILURE',compile,
        reason:'WASM_HOST_CANNOT_VERIFY_STDOUT_OR_TYPED_TRAP'};
    if(t.kind==='observe') {
      const state=classifyUndecidedCompile(compile,existsSync(c.artifact),readdirSync(c.workdir).length>0);
      if(state!==null)return {status:state,compile,reason:'unresolved source form compilation outcome'};
    }
    if(compile.exit_code!==0||!existsSync(c.artifact))return {status:'MISMATCH',compile,reason:'compiler did not produce guest artifact'};
    const artifact_sha256=hash(readFileSync(regular(c.artifact)));
    const guest=invoke(a.run_argv,c);
    if(guest.error||guest.signal||hash(readFileSync(regular(c.artifact)))!==artifact_sha256)return {status:'INFRASTRUCTURE_FAILURE',compile,guest};
    return {status:classifyGuest(t,guest,a.value_mode),compile,guest,artifact_sha256};
  } finally{rmSync(tmp,{recursive:true,force:true});}
}
export function main(args){
  const opts=parseArgs(args);
  let result=audit();
  if(opts.execute){
    if(!isAbsolute(opts.adapter))fail('adapter path must be absolute');
    const m=loadManifest();
    const a=checkAdapter(JSON.parse(readFileSync(regular(opts.adapter),'utf8')),opts.backend,m.cases);
    result={schema:'atto.java_rust.differential.report.v1',mode:'LOCAL_OBSERVATION_NOT_ATTESTATION',cases:m.cases.length,backend:opts.backend,compiler_commit:a.compiler_commit,compiler_tree:a.compiler_tree,guest_parity_certified:false,records:m.cases.map(t=>({case_id:t.id,backend:opts.backend,source_sha256:t.sha256,oracle_kind:t.kind,...runCase(t,a)}))};
  }
  if(opts.execute)assertObservationReport(result);
  const json=JSON.stringify(result,null,2)+'\n';
  if(opts.output){if(!isAbsolute(opts.output))fail('output must be absolute');writeFileSync(opts.output,json,{flag:'wx'});}else process.stdout.write(json);
  // Unattested local observations MUST NOT exit as a green parity validation.
  if(!opts.execute)return 0;
  return result.records.some(x=>!['OBSERVED_UNVERIFIED','OBSERVATION_ONLY'].includes(x.status))?2:3;
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  try{process.exitCode=main(process.argv.slice(2));}
  catch(e){console.error('FAIL CLOSED: '+e.message);process.exitCode=2;}
}
