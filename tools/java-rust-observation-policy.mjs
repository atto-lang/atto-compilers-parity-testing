// Local differential observations are not compiler attestation.
// Pure classification helpers: no shell, file access, network, or execution.
const EXPECTED = new Set(['OBSERVED_UNVERIFIED','OBSERVATION_ONLY','MISMATCH','UNSUPPORTED','INFRASTRUCTURE_FAILURE']);
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;

export function canonicalI64(value) {
  if (typeof value === 'number' && Number.isSafeInteger(value)) return BigInt(value);
  if (typeof value !== 'string' || !/^-?(0|[1-9][0-9]*)$/.test(value)) return null;
  const parsed = BigInt(value);
  return parsed >= I64_MIN && parsed <= I64_MAX ? parsed : null;
}

export function hasDiagnostic(output, code) {
  if (typeof output !== 'string' || typeof code !== 'string' ||
      !/^[A-Z][A-Z0-9_]{2,79}$/.test(code)) return false;
  const re = new RegExp('(?:^|[^A-Za-z0-9_])' + code + '(?:$|[^A-Za-z0-9_])');
  return re.test(output);
}

export function classifyCompileNegative(compile, artifactAbsent, expectedCode) {
  if (compile.error || compile.signal || compile.exit_code === null) return 'INFRASTRUCTURE_FAILURE';
  const diagnostic = (compile.stdout || '') + '\n' + (compile.stderr || '');
  return compile.exit_code !== 0 && artifactAbsent && hasDiagnostic(diagnostic, expectedCode)
    ? 'OBSERVED_UNVERIFIED' : 'MISMATCH';
}

export function classifyUnsupported(compile, artifactAbsent, expectedCode) {
  if (compile.error || compile.signal || compile.exit_code === null) return 'INFRASTRUCTURE_FAILURE';
  const diagnostic = (compile.stdout || '') + '\n' + (compile.stderr || '');
  return compile.exit_code !== 0 && artifactAbsent && hasDiagnostic(diagnostic, expectedCode)
    ? 'UNSUPPORTED' : 'MISMATCH';
}

export function classifyGuest(fixture, guest, valueMode) {
  if (guest.error || guest.signal || guest.exit_code === null) return 'INFRASTRUCTURE_FAILURE';
  if (fixture.kind === 'observe') return 'OBSERVATION_ONLY';
  let correct = false;
  if (fixture.kind === 'stdout') {
    correct = guest.exit_code === fixture.expected_exit && guest.stdout === fixture.expected_stdout;
  } else if (fixture.kind === 'value') {
    if (valueMode === 'stdout_i64') {
      const str = guest.stdout;
      const match = typeof str === 'string' && /^-?(0|[1-9][0-9]*)\n?$/.test(str);
      const expected = canonicalI64(fixture.expected_value);
      correct = guest.exit_code === 0 && match && expected !== null &&
        BigInt(str.trim()) === expected;
    } else if (valueMode === 'exit_code') {
      correct = Number.isInteger(fixture.expected_value) && fixture.expected_value >= 0 &&
        fixture.expected_value <= 255 && guest.exit_code === fixture.expected_value;
    }
  } else if (fixture.kind === 'trap') {
    correct = guest.exit_code !== 0 && hasDiagnostic(guest.stderr || '', fixture.expected_diagnostic);
  }
  return correct ? 'OBSERVED_UNVERIFIED' : 'MISMATCH';
}

export function assertObservationReport(report) {
  if (report.guest_parity_certified !== false || !Array.isArray(report.records))
    throw new Error('observation report cannot certify guest parity');
  for (const entry of report.records) {
    if (!EXPECTED.has(entry.status)) throw new Error('non-observational result status');
    if (entry.status === 'UNSUPPORTED' && !entry.compile) throw new Error('unsupported requires compiled source rejection');
  }
  return report;
}
