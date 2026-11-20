#!/usr/bin/env node
// A restricted host for actual, source-compiled, import-free Atto WebAssembly.
// This standalone guest host is NOT an attestation or an Atto compiler.
import {readFileSync, lstatSync, realpathSync} from 'node:fs';
import {isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';

const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
export function runWasmI64(file) {
  if (typeof file !== 'string' || !isAbsolute(file)) throw new Error('WASM_INPUT_NOT_ABSOLUTE');
  const metadata = lstatSync(file);
  if (!metadata.isFile() || metadata.isSymbolicLink() || realpathSync(file) !== file ||
      metadata.size < 8 || metadata.size > 16 * 1024 * 1024)
    throw new Error('WASM_INPUT_NOT_REGULAR_BOUNDED');
  const bytes = readFileSync(file);
  if (!WebAssembly.validate(bytes)) throw new Error('WASM_MODULE_INVALID');
  const module = new WebAssembly.Module(bytes);
  if (WebAssembly.Module.imports(module).length !== 0) throw new Error('WASM_IMPORTS_NOT_ALLOWED');
  const instance = new WebAssembly.Instance(module, {});
  if (typeof instance.exports.main !== 'function') throw new Error('WASM_MAIN_NOT_EXPORTED');
  let value;
  try { value = instance.exports.main(); }
  catch { throw new Error('WASM_GUEST_TRAP_UNCLASSIFIED'); }
  if (typeof value !== 'bigint' || value < I64_MIN || value > I64_MAX)
    throw new Error('WASM_MAIN_NOT_I64');
  if ('panic_code' in instance.exports) {
    const panic = instance.exports.panic_code;
    if (!(panic instanceof WebAssembly.Global) || (panic.value !== 0 && panic.value !== 0n))
      throw new Error('WASM_PANIC_CODE_NONZERO');
  }
  return value.toString(10);
}

// CLI stays deliberately narrow: one absolute artifact path, decimal stdout.
if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  try {
    if (process.argv.length !== 3) throw new Error('WASM_USAGE_ARTIFACT_REQUIRED');
    process.stdout.write(runWasmI64(process.argv[2]) + '\n');
  } catch (error) {
    console.error('WASM_RUNNER_ERROR: ' + (error?.message || 'unknown failure'));
    process.exitCode = 2;
  }
}
