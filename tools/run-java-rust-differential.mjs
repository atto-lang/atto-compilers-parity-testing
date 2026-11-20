#!/usr/bin/env node
// Opt-in AttoLang Java/Rust candidate observations; NEVER a guest attestation.
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync, mkdtempSync, lstatSync, realpathSync, existsSync, rmSync} from 'node:fs';
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
    seen.add(c.id);
    if(!['canonical','candidate','decision_required'].includes(c.authority)||!['value','stdout','reject','trap','observe'].includes(c.kind))fail('bad oracle');
    if((c.kind==='observe')!==(c.authority==='decision_required'))fail('unreviewed oracle');
    if(!hex64.test(c.sha256)||hash(readFileSync(sourcePath(c.path)))!==c.sha256)fail('fixture hash mismatch: '+c.id);
    if(c.kind==='value'&&!Number.isSafeInteger(c.expected_value))fail('missing integer oracle');
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
  for(let i=0;i<args.length;i++){
    if(args[i]==='--audit')continue;
    if(args[i]==='--execute'){out.execute=true;continue;}
    if(!['--backend','--adapter','--output'].includes(args[i])||!args[i+1])fail('unknown/missing option: '+args[i]);
    out[args[i++].slice(2)]=args[i];
  }
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
function checkAdapter(a,b){
  if(a.backend!==b||!hex40.test(a.compiler_commit)||!hex40.test(a.compiler_tree))fail('unpinned compiler commit/tree');
  if(!hex64.test(a.compiler_sha256)||hash(readFileSync(regular(a.compiler_executable)))!==a.compiler_sha256)fail('compiler binary hash mismatch');
  if(!hex64.test(a.guest_runner_sha256)||hash(readFileSync(regular(a.guest_runner_executable)))!==a.guest_runner_sha256)fail('guest runner hash mismatch');
  checkCommand(a.compile_argv,a.compiler_executable,true);
  checkCommand(a.run_argv,a.guest_runner_executable,false);
  if(!['exit_code','stdout_i64'].includes(a.value_mode))fail('unapproved value extraction');
  if(a.unsupported_cases!==undefined&&(!Array.isArray(a.unsupported_cases)||a.unsupported_cases.some(x=>typeof x!=='string')))fail('bad unsupported list');
  if(a.diagnostic_needles!==undefined&&(!a.diagnostic_needles||typeof a.diagnostic_needles!=='object'||Array.isArray(a.diagnostic_needles)))fail('bad diagnostics');
  return a;
}
function invoke(argv,c){
  const args=argv.map(s=>s.replaceAll('{source}',c.source).replaceAll('{artifact}',c.artifact).replaceAll('{workdir}',c.workdir));
  const p=spawnSync(args[0],args.slice(1),{cwd:c.workdir,shell:false,encoding:'utf8',timeout:15000,maxBuffer:1024*1024,windowsHide:true});
  return {exit_code:p.status,signal:p.signal,stdout:(p.stdout||'').slice(0,4096),stderr:(p.stderr||'').slice(0,4096),error:p.error?.code||null};
}
function needle(t,a){
  const n=a.diagnostic_needles?.[t.id];
  if(typeof n!=='string'||n.length<4||n.length>256||!n.includes(t.expected_diagnostic))fail('missing targeted diagnostic for '+t.id);
  return n;
}
function runCase(t,a){
  if((a.unsupported_cases||[]).includes(t.id))return {status:'UNSUPPORTED'};
  const tmp=mkdtempSync(join(tmpdir(),'atto-diff-'));
  const c={source:sourcePath(t.path),artifact:join(tmp,'guest-artifact'),workdir:tmp};
  try{
    const compile=invoke(a.compile_argv,c);
    if(compile.error||compile.signal)return {status:'INFRASTRUCTURE_FAILURE',compile};
    if(t.kind==='reject'){
      const absent=!existsSync(c.artifact);
      const accepted=compile.exit_code!==0&&(compile.stdout+'\n'+compile.stderr).includes(needle(t,a))&&absent;
      return {status:accepted?'OBSERVED_UNVERIFIED':'MISMATCH',compile,artifact_absent:absent};
    }
    if(compile.exit_code!==0||!existsSync(c.artifact))return {status:'MISMATCH',compile,reason:'compiler did not produce guest artifact'};
    const artifact_sha256=hash(readFileSync(regular(c.artifact)));
    const guest=invoke(a.run_argv,c);
    if(guest.error||guest.signal||hash(readFileSync(regular(c.artifact)))!==artifact_sha256)return {status:'INFRASTRUCTURE_FAILURE',compile,guest};
    if(t.kind==='observe')return {status:'OBSERVATION_ONLY',compile,guest,artifact_sha256};
    let accepted=false;
    if(t.kind==='value'){
      const value=a.value_mode==='exit_code'?guest.exit_code:(/^-?(0|[1-9][0-9]*)\n?$/.test(guest.stdout)?Number(guest.stdout.trim()):null);
      accepted=guest.exit_code!==null&&value===t.expected_value;
    }
    if(t.kind==='stdout')accepted=guest.exit_code===t.expected_exit&&guest.stdout===t.expected_stdout;
    if(t.kind==='trap')accepted=guest.exit_code!==null&&guest.exit_code!==0&&(guest.stdout+'\n'+guest.stderr).includes(needle(t,a));
    return {status:accepted?'OBSERVED_UNVERIFIED':'MISMATCH',compile,guest,artifact_sha256};
  } finally{rmSync(tmp,{recursive:true,force:true});}
}
export function main(args){
  const opts=parseArgs(args);
  let result=audit();
  if(opts.execute){
    if(!isAbsolute(opts.adapter))fail('adapter path must be absolute');
    const a=checkAdapter(JSON.parse(readFileSync(opts.adapter,'utf8')),opts.backend);
    const m=loadManifest();
    result={schema:'atto.java_rust.differential.report.v1',mode:'LOCAL_OBSERVATION_NOT_ATTESTATION',cases:m.cases.length,backend:opts.backend,compiler_commit:a.compiler_commit,compiler_tree:a.compiler_tree,guest_parity_certified:false,records:m.cases.map(t=>({case_id:t.id,backend:opts.backend,source_sha256:t.sha256,oracle_kind:t.kind,...runCase(t,a)}))};
  }
  const json=JSON.stringify(result,null,2)+'\n';
  if(opts.output){if(!isAbsolute(opts.output))fail('output must be absolute');writeFileSync(opts.output,json,{flag:'wx'});}else process.stdout.write(json);
  return opts.execute&&result.records.some(x=>!['OBSERVED_UNVERIFIED','OBSERVATION_ONLY'].includes(x.status))?2:0;
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  try{process.exitCode=main(process.argv.slice(2));}
  catch(e){console.error('FAIL CLOSED: '+e.message);process.exitCode=2;}
}
