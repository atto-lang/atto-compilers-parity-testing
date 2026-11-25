import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';

const manifest = JSON.parse(readFileSync(new URL('../../contracts/target-aware-v1/lanes.json', import.meta.url), 'utf8'));
const lanes = new Map(manifest.lanes.map(l => [l.id, Object.freeze(l)]));
if (lanes.size !== manifest.lanes.length || manifest.schema_version !== 1 ||
    manifest.legacy_wasm_alias !== 'llvm-wasm-bridge' ||
    manifest.lanes.some(l => l.outcome !== 'NOT_EXECUTED')) {
  throw new Error('invalid candidate lane registry');
}
export function lane(id) {
  if (typeof id !== 'string' || !lanes.has(id)) throw new Error('unknown or ambiguous lane');
  return lanes.get(id);
}
const hex = (s, n) => typeof s === 'string' && s.length === n && /^[0-9a-f]+$/.test(s);
const requireHash = (s, n, label) => {
  if (!hex(s, n)) throw new Error('invalid ' + label);
};
export function independentLineages(ids) {
  if (!Array.isArray(ids) || new Set(ids).size !== ids.length) throw new Error('duplicate lane');
  return [...new Set(ids.map(id => lane(id).compiler_lineage).filter(Boolean))].sort();
}
export function i64(value) {
  if (typeof value !== 'string' || value.length > 20 || !/^(0|-?[1-9][0-9]*)$/.test(value))
    throw new Error('expected canonical decimal i64 string');
  const n = BigInt(value);
  if (n < -(1n << 63n) || n > (1n << 63n) - 1n) throw new Error('i64 out of range');
  return n;
}
export function compareValue(expected, observed) {
  if (expected?.kind !== observed?.kind) return false;
  switch (expected.kind) {
    case 'i64': return i64(expected.value) === i64(observed.value);
    case 'error': {
      const valid = v => ['division_by_zero','integer_overflow'].includes(v.code) &&
        Object.keys(v).sort().join(',') === 'code,kind';
      if (!valid(expected) || !valid(observed)) throw new Error('invalid arithmetic error observation');
      return expected.code === observed.code;
    }
    case 'bool':
      if (typeof expected.value !== 'boolean' || typeof observed.value !== 'boolean')
        throw new Error('expected boolean');
      return expected.value === observed.value;
    case 'stdout':
      if (typeof expected.value !== 'string' || typeof observed.value !== 'string')
        throw new Error('expected exact stdout string');
      return expected.value === observed.value;
    default: throw new Error('unknown oracle kind');
  }
}
export const digest = bytes => createHash('sha256').update(bytes).digest('hex');

// Caller supplies policy pins and observations from separate acquisition paths.
// This only compares structure; even matching claims are NOT authenticated receipts.
export function validateCandidate(policy, record) {
  const l = lane(policy.lane);
  if (l.profile !== 'portable' && l.profile !== 'bridge') throw new Error('separate profile verifier required');
  if (record.lane !== l.id || policy.compiler_repo !== l.owner ||
      record.compiler_repo !== l.owner || record.artifact !== l.artifact ||
      record.runtime !== l.runtime || record.mode !== l.mode)
    throw new Error('lane identity mismatch');
  for (const [key, size] of [
    ['compiler_commit',40], ['compiler_tree',40], ['compiler_sha256',64],
    ['source_graph_sha256',64], ['spec_commit',40], ['profile_commit',40],
    ['runtime_sha256',64], ['inputs_sha256',64]
  ]) {
    requireHash(policy[key], size, key);
    requireHash(record[key], size, key);
    if (policy[key] !== record[key]) throw new Error('pin mismatch: ' + key);
  }
  if (record.outcome !== 'OBSERVED' || record.guest_executed !== true ||
      !Number.isSafeInteger(record.executed_steps) || record.executed_steps < 1)
    throw new Error('no executed guest observation');
  requireHash(record.artifact_sha256, 64, 'artifact');
  if (record.artifact_sha256 !== record.artifact_before_run_sha256 ||
      record.artifact_sha256 !== record.artifact_after_run_sha256)
    throw new Error('artifact mismatch');
  if (!compareValue(policy.expected, record.observed)) throw new Error('oracle mismatch');
  return 'CANDIDATE_REQUIRES_INDEPENDENT_PROVENANCE';
}
