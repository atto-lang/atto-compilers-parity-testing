// Static, source-free validator for the *specific* attoc -> Node Wasm host
// command contract. Does not attest any compiler binary or guest result.
import {fileURLToPath} from 'node:url';
const HOST=fileURLToPath(new URL('./run-wasm-i64.mjs',import.meta.url));
const HEX64=/^[0-9a-f]{64}$/;
export function validateWasmAdapterShape(a) {
  if (!a || a.backend !== 'rust-wasm') throw new Error('WASM_BACKEND_MISMATCH');
  if (!Array.isArray(a.compile_argv) || a.compile_argv.length !== 4 ||
      a.compile_argv[0] !== a.compiler_executable || a.compile_argv[1] !== '{source}' ||
      a.compile_argv[2] !== '-o' || a.compile_argv[3] !== '{artifact}')
    throw new Error('WASM_COMPILE_ARGV_UNTRUSTED');
  if (a.guest_runner_script !== HOST || !HEX64.test(a.guest_runner_script_sha256 || '') ||
      !Array.isArray(a.run_argv) || a.run_argv.length !== 3 ||
      a.run_argv[0] !== a.guest_runner_executable ||
      a.run_argv[1] !== HOST || a.run_argv[2] !== '{artifact}' ||
      a.value_mode !== 'stdout_i64') throw new Error('WASM_RUN_ARGV_UNTRUSTED');
  return {guest_runner_script:HOST,guest_runner_script_sha256:a.guest_runner_script_sha256};
}
export const expectedWasmHostPath=HOST;
