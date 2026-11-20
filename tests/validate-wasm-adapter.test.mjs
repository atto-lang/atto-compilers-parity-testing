import {test} from 'node:test';
import assert from 'node:assert/strict';
import {validateWasmAdapterShape,expectedWasmHostPath} from '../tools/guest/validate-wasm-adapter.mjs';
const base=()=>({
  backend:'rust-wasm',compiler_executable:'/trusted/attoc',guest_runner_executable:'/trusted/node',
  guest_runner_script:expectedWasmHostPath,guest_runner_script_sha256:'a'.repeat(64),
  compile_argv:['/trusted/attoc','{source}','-o','{artifact}'],
  run_argv:['/trusted/node',expectedWasmHostPath,'{artifact}'],value_mode:'stdout_i64'
});
test('accept only pinned attoc CLI and canonical embedded wasm guest host',()=>{
 assert.equal(validateWasmAdapterShape(base()).guest_runner_script,expectedWasmHostPath);
 for(const bad of [
  {...base(),backend:'rust-cranelift'},
  {...base(),compile_argv:['/trusted/attoc','{artifact}','-o','{source}']},
  {...base(),compile_argv:['/trusted/attoc','--check','{source}','{artifact}']},
  {...base(),run_argv:['/trusted/node','-e','console.log(42)']},
  {...base(),run_argv:['/trusted/node','/untrusted/fake.mjs','{artifact}']},
  {...base(),guest_runner_script:'/untrusted/fake.mjs'},
  {...base(),guest_runner_script_sha256:'NOT_A_DIGEST'},
  {...base(),value_mode:'exit_code'}
 ]) assert.throws(()=>validateWasmAdapterShape(bad));
});

test('synthetic host return values cannot spoof stdout or typed trap evidence',async()=>{
 const {supportsWasmOracle}=await import('../tools/guest/validate-wasm-adapter.mjs');
 for(const kind of ['value','reject','observe'])assert.equal(supportsWasmOracle(kind),true);
 for(const kind of ['stdout','trap','anything-else'])assert.equal(supportsWasmOracle(kind),false);
});
