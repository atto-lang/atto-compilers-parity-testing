import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

const raw = readFileSync(new URL('../contracts/v1/atto-backend-inventory-v1.json', import.meta.url), 'utf8');
const original = JSON.parse(raw);
const stdio = JSON.parse(readFileSync(new URL('../contracts/v1/stdio-adt-parity-v1.json', import.meta.url), 'utf8'));
const names = ['atto','cranelift','forge','hdl','java','js','llvm','net','realtime','rs','wasm'];
const ids = {atto:1461897456,cranelift:1448702554,forge:1448701874,hdl:1448703717,
  java:1448697198,js:1448703338,llvm:1448700962,net:1448702222,
  realtime:1448704177,rs:1448702852,wasm:1448701355};
const sha = /^[0-9a-f]{40}$/;
function keys(x, fields) {
  return x && typeof x === 'object' && !Array.isArray(x) &&
    Object.keys(x).sort().join('|') === [...fields].sort().join('|');
}
function verify(m) {
  assert.ok(keys(m,['schema','snapshot_date','authority','scope','verification','evidence_rule','backends']));
  assert.equal(m.schema,'atto.source.backend-inventory.v1');
  assert.equal(m.snapshot_date,'2026-11-21');
  assert.equal(m.authority,'GitHub repository, branch and Git-commit metadata');
  assert.equal(m.scope,'11 canonical atto-lang/atto-source.* repositories');
  assert.equal(m.verification,'NOT_CERTIFIED');
  assert.ok(typeof m.evidence_rule === 'string' && m.evidence_rule.includes('zero-step'));
  assert.ok(Array.isArray(m.backends) && m.backends.length===11);
  const seen = new Set();
  for(const b of m.backends) {
    assert.ok(keys(b,['backend','repository','repository_id','profile','source_commit','source_tree','protected_main','guest_parity','ci_observation']));
    assert.ok(Object.hasOwn(ids,b.backend) && !seen.has(b.backend));
    seen.add(b.backend);
    assert.equal(b.repository,'atto-lang/atto-source.'+b.backend);
    assert.equal(b.repository_id,ids[b.backend]);
    assert.match(b.source_commit,sha);
    assert.match(b.source_tree,sha);
    assert.ok(typeof b.profile==='string' && b.profile.length>=3);
    assert.equal(b.protected_main,false);
    assert.equal(b.guest_parity,'NOT_EXECUTED');
    if(b.backend==='atto' || b.backend==='realtime') {
      assert.ok(keys(b.ci_observation,['kind']));
      assert.equal(b.ci_observation.kind,'NO_RUN_SAMPLED');
    } else {
      assert.ok(keys(b.ci_observation,['kind','run_id','runner_id','steps']));
      assert.equal(b.ci_observation.kind,'ZERO_STEP_SAMPLE');
      assert.ok(Number.isSafeInteger(b.ci_observation.run_id) && b.ci_observation.run_id>0);
      assert.equal(b.ci_observation.runner_id,0);
      assert.equal(b.ci_observation.steps,0);
    }
  }
  assert.deepEqual([...seen].sort(),names);
  const other = new Set(stdio.backends.map(x=>x.backend));
  assert.deepEqual([...other].sort(),names);
  return true;
}
test('all eleven exact source identities are only an unverified snapshot',()=>{
  assert.equal(JSON.stringify(original,null,2)+'\n',raw,'noncanonical JSON or duplicate keys');
  assert.ok(verify(original));
});
test('reject omitted, duplicate, foreign and legacy backends',()=>{
  for(let i=0;i<11;i++) {
    const missing=structuredClone(original);missing.backends.splice(i,1);assert.throws(()=>verify(missing));
    const duplicate=structuredClone(original);duplicate.backends[i].backend=duplicate.backends[(i+1)%11].backend;assert.throws(()=>verify(duplicate));
    const foreign=structuredClone(original);foreign.backends[i].repository='ores-truffle-oreslang/oreslang-source.llvm';assert.throws(()=>verify(foreign));
  }
});
test('reject forged guest PASS, head/tree/ID and branch-protection claims',()=>{
  for(let i=0;i<11;i++) for(const [k,v] of [
    ['guest_parity','PASS'],['source_commit','0'.repeat(39)],
    ['source_tree','not-a-git-tree'],['repository_id',1],['protected_main',true]
  ]) {const x=structuredClone(original);x.backends[i][k]=v;assert.throws(()=>verify(x),i+'/'+k);}
  const p=structuredClone(original);p.verification='PASSED';assert.throws(()=>verify(p));
});
test('zero-step jobs cannot be upgraded to passes by source-only edits',()=>{
  for(let i=0;i<11;i++) {
    const extra=structuredClone(original);extra.backends[i].execution_receipt='passed';assert.throws(()=>verify(extra));
    const x=structuredClone(original);
    if(x.backends[i].ci_observation.kind==='ZERO_STEP_SAMPLE') {
      x.backends[i].ci_observation.steps=1;assert.throws(()=>verify(x));
      const y=structuredClone(original);y.backends[i].ci_observation.runner_id=123;assert.throws(()=>verify(y));
    } else {x.backends[i].ci_observation={kind:'ZERO_STEP_SAMPLE',run_id:5,runner_id:0,steps:0};assert.throws(()=>verify(x));}
  }
});
