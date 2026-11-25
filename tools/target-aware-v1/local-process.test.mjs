import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, realpathSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execute, fixtureBytes, fixture } from './local-adapter.mjs';
import { digest } from './validate.mjs';

// Synthetic compiler subprocess tests only. This is not Atto compiler evidence.
const source = `// Candidate guest source: integer truncation and dividend-signed remainder.
// Target main return 42. Not yet compiler/AOT executed.
pub fnc divide(int lhs, int rhs): int {
  return lhs / rhs;
}
pub fnc remainder(int lhs, int rhs): int {
  return Integer.rem(lhs, rhs);
}
pub fnc main(): int {
  val int a = -7;
  val int b = 3;
  return divide(a, b) * -21 + (remainder(a, b) + 1);
}
`;
function setup(t, body, fixtureSource=source, case_id=fixture.id, profile='strict-i64') {
  const dir = realpathSync(mkdtempSync(join(tmpdir(),'atto-synthetic-')));
  t.after(() => rmSync(dir,{recursive:true,force:true}));
  const root = join(dir,'repo'), input = join(dir,'main.atto');
  mkdirSync(join(root,'bin'),{recursive:true});
  writeFileSync(input,fixtureSource);
  const cli = join(root,'bin/oreslang-jsc.mjs');
  writeFileSync(cli,`import {writeFileSync} from 'node:fs';\nconst out=process.argv[4].slice('--out='.length);\n${body}\n`);
  const git = (...args) => execFileSync('git',['-C',root,...args],{encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
  git('init','--quiet'); git('add','.');
  git('-c','user.name=Synthetic Test','-c','user.email=synthetic@example.invalid','commit','--quiet','-m','synthetic compiler fixture');
  return {lane:'js-esm',profile,case_id,root,source:input,commit:git('rev-parse','HEAD'),tree:git('rev-parse','HEAD^{tree}'),
    compiler_sha256:digest(readFileSync(cli)),node_sha256:digest(readFileSync(realpathSync(process.execPath)))};
}
test('synthetic subprocess compiles and executes ESM without certifying Atto',t=>{
  const r=execute(setup(t,`writeFileSync(out,'export function main(){return 42n;}');`));
  assert.equal(r.status,'LOCAL_CANDIDATE_MATCH');
  assert.equal(r.observed.value,'42');
  assert.equal(r.claim,'UNAUTHENTICATED_LOCAL_OBSERVATION_NOT_PARITY_PASS');
});
test('synthetic 298 is not 42 despite identical POSIX exit status',t=>{
  const r=execute(setup(t,`writeFileSync(out,'export function main(){return 298n;}');`));
  assert.equal(r.status,'LOCAL_CANDIDATE_MISMATCH');
  assert.equal(r.observed.value,'298');
});
test('failed synthetic compiler may not leave an executable artifact',t=>{
  assert.throws(()=>execute(setup(t,`writeFileSync(out,'export function main(){return 42n;}');process.exitCode=1;`)),/left an artifact/);
});
test('synthetic compiler failure remains unclassified, not a successful rejection',t=>{
  const r=execute(setup(t,`console.error('fake diagnostic');process.exitCode=1;`));
  assert.equal(r.status,'COMPILE_FAILED_UNCLASSIFIED');
});
test('synthetic guest output injection cannot satisfy the oracle',t=>{
  assert.throws(()=>execute(setup(t,`writeFileSync(out,'export function main(){console.log("fake");return 42n;}');`)),/invalid exact guest observation/);
});

const zeroSource="// Candidate guest source: must fail with division_by_zero (not host exception).\n// Backend may issue a statically proved diagnostic or a typed guest runtime fault.\npub fnc checked(int divisor): int {\n  return 7 / divisor;\n}\npub fnc main(): int {\n  return checked(0);\n}\n";
const errorConfig=(t,body)=>setup(t,body,zeroSource,'reject-i64-zero-divisor','js-program-v2');
test('synthetic structured error matches only an executed root failure with exact identity',t=>{
  const result=execute(errorConfig(t,`if(process.argv[2]!=='--program-v2')throw Error('wrong profile');writeFileSync(out,'export function main(){const e=new RangeError("zero");Object.defineProperties(e,{name:{value:"DivisionByZeroError"},code:{value:"division_by_zero"},abiCode:{value:65538}});throw Object.freeze(e);}');`));
  assert.equal(result.status,'LOCAL_CANDIDATE_MATCH');
  assert.deepEqual(result.observed,{kind:'error',code:'division_by_zero'});
  assert.equal(result.claim,'UNAUTHENTICATED_LOCAL_OBSERVATION_NOT_PARITY_PASS');
});
test('wrong arithmetic identity is mismatch; generic and mutable errors are unclassified',t=>{
  const mismatch=execute(errorConfig(t,`writeFileSync(out,'export function main(){const e=new RangeError("overflow");Object.defineProperties(e,{name:{value:"ArithmeticOverflowError"},code:{value:"integer_overflow"},abiCode:{value:65537}});throw Object.freeze(e);}');`));
  assert.equal(mismatch.status,'LOCAL_CANDIDATE_MISMATCH');
  for(const expr of ['new RangeError("Division by zero")','Object.assign(new RangeError("zero"),{name:"DivisionByZeroError",code:"division_by_zero",abiCode:65538})','Object.freeze(Object.assign(new Error("zero"),{name:"DivisionByZeroError",code:"division_by_zero",abiCode:65538}))']){
    const r=execute(errorConfig(t,`writeFileSync(out,${JSON.stringify('export function main(){throw '+expr+';}')});`));
    assert.equal(r.status,'GUEST_FAILED_UNCLASSIFIED');
  }
});
test('compiler failure cannot satisfy expected arithmetic failure',t=>{
  const r=execute(errorConfig(t,`console.error("division_by_zero");process.exitCode=1;`));
  assert.equal(r.status,'COMPILE_FAILED_UNCLASSIFIED');
});
