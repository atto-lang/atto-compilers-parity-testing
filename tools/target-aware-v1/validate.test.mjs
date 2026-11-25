import test from 'node:test';
import assert from 'node:assert/strict';
import { lane, independentLineages, i64, compareValue, validateCandidate } from './validate.mjs';

const pins = {
  compiler_commit: 'a'.repeat(40), compiler_tree: 'b'.repeat(40),
  compiler_sha256: 'c'.repeat(64), source_graph_sha256: 'd'.repeat(64),
  spec_commit: 'e'.repeat(40), profile_commit: 'f'.repeat(40),
  runtime_sha256: '1'.repeat(64), inputs_sha256: '2'.repeat(64)
};
function pair(id = 'rs-wasm') {
  const l = lane(id);
  const expected = {kind:'i64', value:'9007199254740993'};
  const policy = {...pins, lane:id, compiler_repo:l.owner, expected};
  const hash = '3'.repeat(64);
  const record = {...pins, lane:id, compiler_repo:l.owner, artifact:l.artifact,
    runtime:l.runtime, mode:l.mode, outcome:'OBSERVED', guest_executed:true,
    executed_steps:2, artifact_sha256:hash, artifact_before_run_sha256:hash,
    artifact_after_run_sha256:hash, observed:{...expected}};
  return [policy, record];
}

test('Wasm lane identity is explicit, not a second independent compiler', () => {
  assert.equal(lane('rs-wasm-worker').owner, lane('rs-wasm').owner);
  assert.notEqual(lane('llvm-wasm-bridge').owner, lane('rs-wasm').owner);
  assert.throws(() => lane('wasm'), /ambiguous/);
  assert.equal(independentLineages(['rs-wasm','rs-wasm-worker','llvm-wasm-bridge']).length, 2);
  assert.equal(independentLineages(['java-jvm','java-native']).length, 1);
  assert.equal(independentLineages(['atto-bootstrap','realtime-profile']).length, 0);
  assert.throws(() => independentLineages(['rs-wasm','rs-wasm']), /duplicate/);
});

test('integer comparisons preserve full signed width', () => {
  for (const [a,b] of [['42','298'],['-1','255'],['9007199254740993','9007199254740992']])
    assert.equal(compareValue({kind:'i64',value:a},{kind:'i64',value:b}),false);
  assert.equal(i64('-9223372036854775808'), -(1n<<63n));
  assert.equal(i64('9223372036854775807'), (1n<<63n)-1n);
  for (const x of ['9223372036854775808','-9223372036854775809','01','-0','+1','1e3',' 1',1])
    assert.throws(() => i64(x));
});

test('stdout and booleans are exact and type sensitive', () => {
  assert.equal(compareValue({kind:'stdout',value:'42\n'},{kind:'stdout',value:'42\nextra'}),false);
  assert.equal(compareValue({kind:'bool',value:true},{kind:'bool',value:false}),false);
  assert.throws(() => compareValue({kind:'bool',value:true},{kind:'bool',value:1}));
});

test('matching observation remains a candidate, never PASS', () => {
  for (const id of ['rs-wasm','rs-wasm-worker','java-jvm','java-native','net-cil','net-native',
    'llvm-native','llvm-wasm-bridge','js-esm','forge-native','cranelift-jit','cranelift-aot'])
    assert.equal(validateCandidate(...pair(id)), 'CANDIDATE_REQUIRES_INDEPENDENT_PROVENANCE');
});

test('no compiler or target substitution', () => {
  for (const [id, patch] of [
    ['rs-wasm',{compiler_repo:'atto-lang/atto-source.llvm'}],
    ['rs-wasm-worker',{compiler_repo:'atto-lang/atto-source.wasm'}],
    ['js-esm',{mode:'aot'}], ['net-native',{artifact:'cil'}],
    ['java-native',{runtime:'jvm'}], ['rs-wasm',{lane:'rs-wasm-worker'}]
  ]) {
    const [p,r] = pair(id);
    assert.throws(() => validateCandidate(p,{...r,...patch}), /lane identity/);
  }
});

test('reject mismatched provenance pins and changed artifact', () => {
  const [p,r] = pair();
  for (const key of Object.keys(pins)) {
    assert.throws(() => validateCandidate(p,{...r,[key]:'9'.repeat(pins[key].length)}), /pin mismatch/);
    assert.throws(() => validateCandidate(p,{...r,[key]:'bad'}), /invalid/);
  }
  assert.throws(() => validateCandidate(p,{...r,artifact_after_run_sha256:'9'.repeat(64)}), /artifact mismatch/);
});

test('no skipped, host-only, unsupported, or malformed guest success', () => {
  const [p,r] = pair();
  for (const patch of [{executed_steps:0},{executed_steps:1.5},{guest_executed:false},
    {outcome:'PASS'},{outcome:'UNSUPPORTED'},{outcome:'NOT_EXECUTED'}])
    assert.throws(() => validateCandidate(p,{...r,...patch}), /no executed/);
  assert.throws(() => validateCandidate(p,{...r,observed:{kind:'i64',value:'42'}}), /oracle mismatch/);
});

test('bootstrap, HDL and realtime do not get portable guest certification', () => {
  for (const id of ['atto-bootstrap','hdl-rtl','realtime-profile'])
    assert.throws(() => validateCandidate(...pair(id)), /separate profile/);
});

test('arithmetic error candidates are closed and never authenticate an execution receipt',()=>{
  const expected={kind:'error',code:'division_by_zero'};
  assert.equal(compareValue(expected,{...expected}),true);
  assert.equal(compareValue(expected,{kind:'error',code:'integer_overflow'}),false);
  assert.equal(compareValue(expected,{kind:'i64',value:'42'}),false);
  for(const invalid of [{kind:'error',code:'panic'},{...expected,abiCode:65538}])
    assert.throws(()=>compareValue(expected,invalid),/invalid arithmetic error/);
  const [p,r]=pair('js-esm');
  assert.equal(validateCandidate({...p,expected},{...r,observed:expected}),'CANDIDATE_REQUIRES_INDEPENDENT_PROVENANCE');
  assert.throws(()=>validateCandidate({...p,expected},{...r,observed:expected,guest_executed:false}),/no executed/);
});
