#!/usr/bin/env node
// Cross-backend std/env receipt sanity check. NEVER authenticates GitHub Actions
// or asserts a self-reported compiled-guest PASS as independently verified.
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {dirname, resolve, join} from 'node:path';
import {fileURLToPath} from 'node:url';

const HERE=dirname(fileURLToPath(import.meta.url));
export const BACKENDS=['java','llvm','net','rs','forge','cranelift','js','wasm','atto','realtime','hdl'];
export const CASES=['basic-values','strict-bool','invalid-bool',
  'required-missing-empty','denied-fallback','denied-probe','coarse-denied',
  'invalid-name','oversized-value','host-ambient-hidden',
  'independent-contexts','package-no-shadow','present-empty-bool-fails'];
// Immutable independent source-contract review target; do not use a PR-local
// override to accept an invented or downgraded guest test suite.
export const CONTRACT_BLOB='e20331b2c17e29eefdd2ba04ea5f750c0b56eff1';
const HEX40=/^[a-f0-9]{40}$/;
const RUN_URL=/^https:\/\/github\.com\/[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+\/actions\/runs\/[1-9][0-9]*$/;
const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
const gitBlob=content=>createHash('sha1').update('blob '+content.length+'\0').update(content).digest('hex');
const strictSet=(have,want)=>Array.isArray(have)&&have.length===want.length&&
  new Set(have).size===want.length&&want.every(x=>have.includes(x));
const plain=o=>o!==null&&typeof o==='object'&&!Array.isArray(o);

export function validateStdEnvLedger(ledger,contract=null) {
  const errors=[];
  const fail=x=>errors.push(x);
  if (!plain(ledger)) return {ok:false,errors:['not a receipt ledger'],claim:'INVALID'};
  if (ledger.schema!=='atto-std-env-receipt-v1'||ledger.feature!=='std/env'||
      ledger.claim!=='UNVERIFIED_GUEST_PARITY'||
      ledger.contract_git_blob_sha!==CONTRACT_BLOB)
    fail('invalid schema/contract identity or forged claim');
  if (!Array.isArray(ledger.targets)||ledger.targets.length!==BACKENDS.length)
    fail('all eleven backends must be explicit');
  let requiresContract=false;
  const names=new Set();
  for(const entry of Array.isArray(ledger.targets)?ledger.targets:[]) {
    if (!plain(entry)||!BACKENDS.includes(entry.backend)) {
      fail('invalid backend record');continue;
    }
    if (names.has(entry.backend)) fail('duplicate backend '+entry.backend);
    names.add(entry.backend);
    if (!['not_executed','unsupported','passed','failed'].includes(entry.status)) {
      fail('invalid status '+entry.backend);continue;
    }
    if (entry.status==='not_executed'||entry.status==='unsupported') {
      if (typeof entry.reason!=='string'||entry.reason.length<15||entry.reason.length>512)
        fail('missing bounded reason '+entry.backend);
      for(const property of ['source_sha','tree_sha','run_url','executed_job_steps',
                             'target','toolchain','cases','execution_kind','verified'])
        if (Object.hasOwn(entry,property)) fail('nonexecuted backend claims execution '+entry.backend);
      continue;
    }
    requiresContract=true;
    if (entry.execution_kind!=='compiled_atto_guest') fail('not compiled guest execution '+entry.backend);
    if (!HEX40.test(entry.source_sha??'')||!HEX40.test(entry.tree_sha??'')||
        !RUN_URL.test(entry.run_url??'')||!Number.isInteger(entry.executed_job_steps)||
        entry.executed_job_steps<4||!entry.toolchain||typeof entry.toolchain!=='string'||
        !entry.target||typeof entry.target!=='string')
      fail('missing exact-head, toolchain or stepful CI provenance '+entry.backend);
    if (!Array.isArray(entry.cases)||entry.cases.length!==CASES.length||
        !strictSet((entry.cases??[]).map(c=>c?.id),CASES)) {
      fail('missing/duplicate case records '+entry.backend);continue;
    }
    if (contract) {
      for (const receiptCase of entry.cases) {
        const expected=contract.cases.find(c=>c.id===receiptCase.id);
        if (!expected) {fail('unrecognized case '+receiptCase.id);continue;}
        const hash=contract.fixture_git_blobs?.[expected.fixture];
        if (receiptCase.fixture_blob_sha!==hash || !HEX40.test(hash??''))
          fail('wrong fixture blob '+receiptCase.id);
        if (receiptCase.execution_target!=='guest')
          fail('host-only evidence cannot pass guest case '+receiptCase.id);
        if (receiptCase.compile_executed!==true)
          fail('source-only fixture claim '+receiptCase.id);
        if (receiptCase.execution_kind==='synthetic_host')
          fail('synthetic guest claim '+receiptCase.id);
        if (expected.expected.kind==='admission_error') {
          if (receiptCase.guest_executed!==false ||
              receiptCase.admission_executed!==true ||
              receiptCase.error_category!==expected.expected.category ||
              !same(receiptCase.stdout_lines,[]))
            fail('wrong pre-guest admission error '+receiptCase.id);
        } else {
          if (receiptCase.guest_executed!==true)
            fail('no guest execution '+receiptCase.id);
          if (expected.expected.kind==='stdout' &&
              (!same(receiptCase.stdout_lines,expected.expected.lines) ||
               receiptCase.error_category!==null))
            fail('stdout mismatch '+receiptCase.id);
          if (expected.expected.kind==='stdout_sequence' &&
              (!same(receiptCase.stdout_sequence,expected.expected.runs) ||
               receiptCase.error_category!==null))
            fail('context sequence mismatch '+receiptCase.id);
          if (expected.expected.kind==='runtime_error' &&
              (receiptCase.error_category!==expected.expected.category ||
               !same(receiptCase.stdout_lines,[])))
            fail('runtime error mismatch '+receiptCase.id);
        }
        if ('error_message' in receiptCase || 'raw_stderr' in receiptCase ||
            'raw_environment' in receiptCase)
          fail('raw secret-bearing output forbidden '+receiptCase.id);
      }
    }
  }
  if (!strictSet([...names],BACKENDS)) fail('missing backend identity');
  if (contract && (contract.schema!=='atto-std-env-guest-v1' ||
                   !strictSet(contract.cases?.map(c=>c.id),CASES)))
    fail('wrong case contract');
  if (requiresContract&&!contract)
    fail('receipt claims require independently pinned full contract');
  return {
    ok:errors.length===0,errors,
    claim:errors.length?'INVALID':
      requiresContract?'CANDIDATE_REQUIRES_INDEPENDENT_GITHUB_PROVENANCE':'NOT_EXECUTED'
  };
}

export function loadContract(path) {
  const bytes=readFileSync(path);
  if(gitBlob(bytes)!==CONTRACT_BLOB) throw Error('contract blob SHA mismatch');
  return JSON.parse(bytes.toString('utf8'));
}

if(process.argv[1] && resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  const ledgerPath=resolve(process.argv[2]??join(HERE,'../../contracts/std-env-v1/status-ledger.json'));
  const contract=process.argv[3]?loadContract(resolve(process.argv[3])):null;
  const result=validateStdEnvLedger(JSON.parse(readFileSync(ledgerPath,'utf8')),contract);
  if (!result.ok) {for(const e of result.errors) console.error('FAIL '+e);process.exitCode=1;}
  else console.log('STD_ENV_V1_LEDGER='+result.claim+' backends=11');
  if (result.claim==='CANDIDATE_REQUIRES_INDEPENDENT_GITHUB_PROVENANCE')
    console.log('NOT_ATTESTED: verify GitHub run/job steps and exact source tree independently');
}
