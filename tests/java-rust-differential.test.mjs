import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {loadManifest,audit,parseArgs} from '../tools/run-java-rust-differential.mjs';

const root=fileURLToPath(new URL('../',import.meta.url));
const runner=join(root,'tools/run-java-rust-differential.mjs');

test('all ten distinct fixtures have verified SHA256 and typed oracle',()=>{
  const m=loadManifest();
  assert.equal(m.cases.length,10);
  assert.deepEqual(m.scope.required_backends,['java-graal','rust-wasm','rust-cranelift']);
  assert.equal(new Set(m.cases.map(c=>c.id)).size,10);
  assert.equal(m.cases.filter(c=>c.authority==='decision_required').length,3);
  assert.equal(m.cases.filter(c=>c.status!=='AUTHORED_NOT_EXECUTED').length,0);
});
test('exactly 30 backend cells remain NOT_EXECUTED and uncertified',()=>{
  const d=audit();
  assert.equal(d.mode,'METADATA_ONLY');
  assert.equal(d.guest_parity_certified,false);
  assert.equal(d.records.length,30);
  for(const r of d.records){
    assert.equal(r.status,'NOT_EXECUTED');
    assert.equal(r.guest_executed,false);
    assert.equal(r.executed_steps,0);
    assert.equal(r.provenance,null);
  }
});
test('unconfigured or invalid execute mode fails closed',()=>{
  assert.deepEqual(parseArgs([]),{execute:false,backend:null,adapter:null,output:null});
  for(const args of [['--execute'],['--backend','java-graal'],['--execute','--backend','java-graal'],['--execute','--backend','java'],['--extra']]){
    assert.throws(()=>parseArgs(args));
  }
  const p=spawnSync(process.execPath,[runner,'--execute','--backend','java-graal','--adapter','relative.json'],{encoding:'utf8'});
  assert.equal(p.status,2);
  assert.match(p.stderr,/adapter path must be absolute/);
});
test('CLI audit requires no compiler and reports zero guest claims',()=>{
  const p=spawnSync(process.execPath,[runner,'--audit'],{encoding:'utf8'});
  assert.equal(p.status,0,p.stderr);
  const d=JSON.parse(p.stdout);
  assert.equal(d.cases,10);
  assert.equal(d.records.length,30);
  assert.equal(d.guest_parity_certified,false);
  assert.equal(d.mode,'METADATA_ONLY');
});
test('forged compiler binary digest fails before any compile invocation',()=>{
  const dir=mkdtempSync(join(tmpdir(),'atto-fail-closed-'));
  try {
    const config=join(dir,'adapter.json');
    const fake={
      backend:'java-graal',compiler_commit:'a'.repeat(40),compiler_tree:'b'.repeat(40),
      compiler_executable:process.execPath,compiler_sha256:'0'.repeat(64),
      guest_runner_executable:process.execPath,guest_runner_sha256:'0'.repeat(64),
      compile_argv:[process.execPath,'{source}','{artifact}'],
      run_argv:[process.execPath,'{artifact}'],value_mode:'stdout_i64'
    };
    writeFileSync(config,JSON.stringify(fake));
    const p=spawnSync(process.execPath,[runner,'--execute','--backend','java-graal','--adapter',config],{encoding:'utf8'});
    assert.equal(p.status,2);
    assert.match(p.stderr,/compiler binary hash mismatch/);
    assert.doesNotMatch(p.stdout,/OBSERVED_UNVERIFIED/);
  } finally{rmSync(dir,{force:true,recursive:true});}
});
test('all semantic-decision cases are observation-only, never normative passes',()=>{
  const m=JSON.parse(readFileSync(join(root,'contracts/java-rust-conflicts-v1/manifest.json'),'utf8'));
  assert.equal(m.cases.filter(c=>c.kind==='observe').length,3);
  assert.equal(m.baseline.filter(c=>c.status!=='NOT_EXECUTED').length,0);
  assert.equal(m.cases.filter(c=>c.kind==='trap').length,1);
  assert.equal(m.cases.filter(c=>c.kind==='reject').length,2);
});
