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
function setup(t, body) {
  const dir = realpathSync(mkdtempSync(join(tmpdir(),'atto-synthetic-')));
  t.after(() => rmSync(dir,{recursive:true,force:true}));
  const root = join(dir,'repo'), input = join(dir,'main.atto');
  mkdirSync(join(root,'bin'),{recursive:true});
  writeFileSync(input,source);
  fixtureBytes(input,fixture.blob);
  const cli = join(root,'bin/oreslang-jsc.mjs');
  writeFileSync(cli,`import {writeFileSync} from 'node:fs';\nconst out=process.argv[4].slice('--out='.length);\n${body}\n`);
  const git = (...args) => execFileSync('git',['-C',root,...args],{encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
  git('init','--quiet'); git('add','.');
  git('-c','user.name=Synthetic Test','-c','user.email=synthetic@example.invalid','commit','--quiet','-m','synthetic compiler fixture');
  return {lane:'js-esm',root,source:input,commit:git('rev-parse','HEAD'),tree:git('rev-parse','HEAD^{tree}'),
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
