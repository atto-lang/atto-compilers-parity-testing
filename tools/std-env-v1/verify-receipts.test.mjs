import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname,resolve} from 'node:path';
import {
  validateStdEnvLedger, BACKENDS, CASES, CONTRACT_BLOB
} from './verify-receipts.mjs';

const HERE=dirname(fileURLToPath(import.meta.url));
const original=JSON.parse(readFileSync(resolve(HERE,'../../contracts/std-env-v1/status-ledger.json'),'utf8'));
const copy=()=>structuredClone(original);
const reject=(ledger,regexp,contract=null)=>{
  const v=validateStdEnvLedger(ledger,contract);
  assert.equal(v.ok,false);
  assert.equal(v.claim,'INVALID');
  assert.match(v.errors.join(' | '),regexp);
};

test('11 backend records are explicitly not executed, not a guest pass',()=>{
  const result=validateStdEnvLedger(copy());
  assert.deepEqual(result,{ok:true,errors:[],claim:'NOT_EXECUTED'});
  assert.equal(original.contract_git_blob_sha,CONTRACT_BLOB);
  assert.equal(original.targets.length,BACKENDS.length);
  assert.equal(CASES.length,13);
});

test('fake PASS and invented GitHub URLs cannot make the source ledger green',()=>{
  const c=copy();c.targets[0].status='passed';
  reject(c,/missing exact-head|missing\/duplicate case records|require independently pinned/);
  const x=copy();x.targets[0].run_url='https://example.test/actions/runs/8';
  reject(x,/nonexecuted backend claims execution/);
});

test('zero-step, host-only and no toolchain claims are rejected',()=>{
  const c=copy();
  Object.assign(c.targets[0],{
    status:'passed',source_sha:'1'.repeat(40),tree_sha:'2'.repeat(40),
    run_url:'https://github.com/org/repo/actions/runs/5',
    execution_kind:'host_only',executed_job_steps:0,toolchain:'',target:'linux-x64',
    cases:[]
  });
  reject(c,/not compiled guest execution|missing exact-head/);
});

test('missing, duplicated, and unknown backend records fail closed',()=>{
  const c=copy();c.targets.pop();reject(c,/all eleven backends/);
  const d=copy();d.targets[1].backend='java';reject(d,/duplicate backend|missing backend identity/);
  const e=copy();e.targets[3].status='green';reject(e,/invalid status/);
});

test('contract identity and explicit claim ceiling are immutable',()=>{
  const c=copy();c.contract_git_blob_sha='0'.repeat(40);
  reject(c,/invalid schema\/contract identity/);
  const d=copy();d.claim='PASSED_ALL_BACKENDS';
  reject(d,/invalid schema\/contract identity/);
});

test('candidate compiled guest report is never authenticated as a PASS',()=>{
  const contract={schema:'atto-std-env-guest-v1',
    fixture_git_blobs:{'fixture.atto':'a'.repeat(40)},
    cases:CASES.map(id=>({id,fixture:'fixture.atto',
      expected:{kind:'stdout',lines:['true']}}))};
  const c=copy();
  const records=CASES.map(id=>({
    id,fixture_blob_sha:'a'.repeat(40),execution_target:'guest',
    compile_executed:true,guest_executed:true,stdout_lines:['true'],
    error_category:null
  }));
  c.targets[0]={
    backend:'java',status:'passed',
    execution_kind:'compiled_atto_guest',
    source_sha:'1'.repeat(40),tree_sha:'2'.repeat(40),
    run_url:'https://github.com/atto-lang/atto-source.java/actions/runs/1234',
    executed_job_steps:10,toolchain:'GraalVM 25',target:'linux-x64',
    cases:records
  };
  assert.deepEqual(validateStdEnvLedger(c,contract),{
    ok:true,errors:[],claim:'CANDIDATE_REQUIRES_INDEPENDENT_GITHUB_PROVENANCE'
  });
  c.targets[0].cases[0].guest_executed=false;
  reject(c,/no guest execution/,contract);
  c.targets[0].cases[0].guest_executed=true;
  c.targets[0].cases[0].raw_stderr='secret';
  reject(c,/raw secret-bearing output forbidden/,contract);
});

test('different case/output or fixture SHA cannot masquerade as guest parity',()=>{
  const contract={schema:'atto-std-env-guest-v1',
    fixture_git_blobs:{'fixture.atto':'a'.repeat(40)},
    cases:CASES.map(id=>({id,fixture:'fixture.atto',
      expected:{kind:'stdout',lines:['ok']}}))};
  const c=copy();c.targets[0]={
    backend:'llvm',status:'passed',execution_kind:'compiled_atto_guest',
    source_sha:'1'.repeat(40),tree_sha:'2'.repeat(40),
    run_url:'https://github.com/atto-lang/atto-source.llvm/actions/runs/1234',
    executed_job_steps:5,toolchain:'clang-19',target:'linux-x64',
    cases:CASES.map(id=>({
      id,fixture_blob_sha:'b'.repeat(40),execution_target:'guest',
      compile_executed:true,guest_executed:true,stdout_lines:['ok'],error_category:null
    }))
  };
  reject(c,/wrong fixture blob/,contract);
  for(const entry of c.targets[0].cases)entry.fixture_blob_sha='a'.repeat(40);
  c.targets[0].cases[0].stdout_lines=['forged'];
  reject(c,/stdout mismatch/,contract);
});

test('unverified candidate cannot omit independently pinned full contract',()=>{
  const c=copy();c.targets[0]={
    backend:'java',status:'passed',execution_kind:'compiled_atto_guest',
    source_sha:'1'.repeat(40),tree_sha:'2'.repeat(40),
    run_url:'https://github.com/atto-lang/atto-source.java/actions/runs/45',
    executed_job_steps:8,toolchain:'GraalVM 25',target:'linux-x64',
    cases:CASES.map(id=>({id}))
  };
  reject(c,/receipt claims require independently pinned full contract/);
});
