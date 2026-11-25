import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, symlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { decodeObservation, fixtureBytes, execute, selectFixture } from './local-adapter.mjs';

test('lossless subprocess observation, not process exit status', () => {
  for (const value of ['42','298','-1','255','9007199254740993','-9223372036854775808'])
    assert.deepEqual(decodeObservation(JSON.stringify({kind:'i64',value})+'\n'),{kind:'i64',value});
});
test('reject output injection, truncation, out-of-range and numeric JSON', () => {
  for (const text of [
    '{"kind":"i64","value":"42"}', '{"kind":"i64","value":"42"}\n\n',
    '{"kind":"i64","value":"42"}\nextra', '{"kind":"i64","value":42}\n',
    '{"kind":"i64","value":"9223372036854775808"}\n', '{"kind":"i64","value":"-0"}\n'
  ]) assert.throws(() => decodeObservation(text));
});
test('fixture pin checks exact Git blob, not merely text equivalence', () => {
  const dir=mkdtempSync(join(tmpdir(),'atto-fixture-test-'));
  try {
    const path=join(dir,'main.atto'), link=join(dir,'link.atto');
    const bytes=Buffer.from('pub fnc main(): int { return 42; }\n');
    const blob=createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex');
    writeFileSync(path,bytes);
    assert.deepEqual(fixtureBytes(path,blob),bytes);
    symlinkSync(path,link);
    assert.throws(() => fixtureBytes(link,blob),/regular file/);
    writeFileSync(path,Buffer.concat([bytes,Buffer.from('\n')]));
    assert.throws(() => fixtureBytes(path,blob),/blob mismatch/);
  } finally { rmSync(dir,{recursive:true,force:true}); }
});
test('unimplemented or ambiguous lanes cannot fall back to a different compiler', () => {
  for (const lane of ['wasm','java-native','llvm-native','rs-wasm-worker','net-native','forge-native'])
    assert.throws(() => execute({lane}),/not implemented/);
});

test('eight shared numeric positive cases are selectable without changing expected source pins', () => {
  for (const id of ['checked-i64-quotient','trap-division-by-zero','trap-integer-overflow',
    'trap-result-no-doublewrap','ordered-float-nan','ordered-float-signed-zero',
    'ordered-int-float-precision','equality-numeric-exact']) {
    const c = selectFixture(id);
    assert.equal(c.id,id);
    assert.equal(c.expected.value,'42');
    assert.equal(c.status,'NOT_EXECUTED');
  }
  assert.equal(selectFixture('ordered-int-float-precision').blob,'2d57f49dd16760f94bef1525b95dfc20ba654a9a');
});
test('negative numeric cases need a structured failure oracle, not a generic compiler failure', () => {
  for (const id of ['reject-i64-zero-divisor','reject-i64-min-overflow','reject-i64-zero-remainder'])
    assert.throws(() => selectFixture(id), /structured failure oracle/);
  assert.throws(() => selectFixture('unknown'), /unknown numeric fixture/);
});

test('closed structured error observations reject aliases, extra fields and injected output',()=>{
  for(const code of ['integer_overflow','division_by_zero'])
    assert.deepEqual(decodeObservation(JSON.stringify({kind:'error',code})+'\n'),{kind:'error',code});
  for(const output of ['{"kind":"error","code":"panic"}\n','{"kind":"error","code":"division_by_zero","extra":1}\n',
    '{"kind":"error","code":"division_by_zero"}\n\n','{"code":"division_by_zero","kind":"error"}\n'])
    assert.throws(()=>decodeObservation(output),/invalid exact guest observation/);
});
test('structured error cases require explicit JS program-v2 profile',()=>{
  for(const id of ['reject-i64-zero-divisor','reject-i64-min-overflow','reject-i64-zero-remainder'])
    assert.equal(selectFixture(id,'js-program-v2').expected.kind,'error');
  assert.throws(()=>execute({lane:'rs-wasm',profile:'js-program-v2'}),/unsupported lane\/profile/);
});
