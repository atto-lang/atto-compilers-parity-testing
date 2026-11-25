import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, symlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { decodeObservation, fixtureBytes, execute } from './local-adapter.mjs';

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
