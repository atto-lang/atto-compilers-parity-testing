import {test} from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync,rmSync,writeFileSync,symlinkSync,readFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {validateNativeAdapterShape,supportsNativeOracle,validateStaticElfArtifact} from '../tools/guest/validate-native-adapter.mjs';

const base=()=>({
  backend:'rust-cranelift',compiler_executable:'/trusted/oresc',
  guest_runner_executable:'/usr/bin/prlimit',guest_runner_sha256:'a'.repeat(64),
  native_sandbox_executable:'/usr/bin/bwrap',native_sandbox_sha256:'c'.repeat(64),
  linker_executable:'/trusted/cc',linker_sha256:'b'.repeat(64),
  compile_argv:['/trusted/oresc','--strict-aot','--checked-i64','{source}','{object}'],
  link_argv:['/trusted/cc','-static','{object}','-o','{artifact}'],
  run_argv:['/usr/bin/prlimit','--as=536870912','--cpu=5','--nproc=32',
    '--nofile=64','--fsize=1048576','--core=0','--','/usr/bin/bwrap','--unshare-all','--die-with-parent','--new-session','--clearenv',
    '--tmpfs','/','--ro-bind','{artifact}','/guest','--dev','/dev','--proc','/proc','--','/guest'],
  value_mode:'exit_code'
});

test('native adapter accepts only exact checked compiler, static linker and isolated guest',()=>{
  assert.equal(validateNativeAdapterShape(base()).linker_executable,'/trusted/cc');
});
test('rejects opt-out checked arithmetic, substitution and shell injection',()=>{
  for(const bad of [
    {...base(),compile_argv:['/trusted/oresc','{source}','{object}']},
    {...base(),compile_argv:[...base().compile_argv,'--shell']},
    {...base(),compile_argv:['/trusted/oresc','--strict-aot','--checked-i64','{artifact}','{object}']},
    {...base(),link_argv:['/trusted/cc','{object}','-o','{artifact}']},
    {...base(),link_argv:['/trusted/cc','-static','{object}','-o','/tmp/fake']},
    {...base(),linker_sha256:'not-a-hash'},
    {...base(),native_host_wrapper:'/tmp/override.sh'}]) assert.throws(()=>validateNativeAdapterShape(bad));
});
test('rejects native execution without isolated bubblewrap command',()=>{
  for(const bad of [
    {...base(),guest_runner_executable:'/tmp/native-binary'},
    {...base(),native_sandbox_executable:'/tmp/fake-sandbox'},
    {...base(),native_sandbox_sha256:'bad-digest'},
    {...base(),run_argv:[...base().run_argv.slice(0,1),'--as=unlimited',...base().run_argv.slice(2)]},
    {...base(),run_argv:['{artifact}']},
    {...base(),run_argv:['/usr/bin/bwrap','--ro-bind','/','/','--','{artifact}']},
    {...base(),run_argv:[...base().run_argv,'--unshare-net']},
    {...base(),value_mode:'stdout_i64'},
    {...base(),backend:'rust-wasm'}])assert.throws(()=>validateNativeAdapterShape(bad));
});
test('only small exit-code value oracle can be observed by this native ABI',()=>{
  for(const expected_value of [0,42,125])assert.equal(supportsNativeOracle({kind:'value',expected_value}),true);
  for(const expected_value of [-1,126,139,255,256,'42','9223372036854775807',null])assert.equal(supportsNativeOracle({kind:'value',expected_value}),false);
  assert.equal(supportsNativeOracle({kind:'reject'}),true);
  assert.equal(supportsNativeOracle({kind:'observe'}),true);
  assert.equal(supportsNativeOracle({kind:'stdout'}),false);
  assert.equal(supportsNativeOracle({kind:'trap'}),false);
});
test('inspect a source-free C-compiled static ELF without executing guest code',()=>{
  const dir=mkdtempSync(join(tmpdir(),'atto-native-elf-'));
  try{
    const src=join(dir,'sample.c'),exe=join(dir,'sample');
    writeFileSync(src,'int main(void) {return 42;}\n');
    const cc=spawnSync('cc',['-static',src,'-o',exe],{encoding:'utf8',timeout:20_000});
    assert.ifError(cc.error);
    // Missing static system CRT is a failed test, not an implicit pass.
    assert.equal(cc.status,0,cc.stderr);
    assert.equal(validateStaticElfArtifact(exe),true);
    const alias=join(dir,'alias');symlinkSync(exe,alias);
    assert.throws(()=>validateStaticElfArtifact(alias),/NATIVE_ARTIFACT_NOT_BOUNDED_REGULAR/);
    const invalid=join(dir,'invalid');writeFileSync(invalid,readFileSync(exe).subarray(0,63));
    assert.throws(()=>validateStaticElfArtifact(invalid),/NATIVE_ARTIFACT_NOT_BOUNDED_REGULAR/);
  }finally{rmSync(dir,{recursive:true,force:true});}
});
test('reject dynamic ELF with PT_INTERP and rejects malformed file',()=>{
  const dir=mkdtempSync(join(tmpdir(),'atto-native-dynamic-'));
  try{
    const src=join(dir,'sample.c'),exe=join(dir,'dynamic');
    writeFileSync(src,'int main(void) {return 42;}\n');
    const cc=spawnSync('cc',[src,'-o',exe],{encoding:'utf8',timeout:20_000});
    assert.ifError(cc.error);
    assert.equal(cc.status,0,cc.stderr);
    assert.throws(()=>validateStaticElfArtifact(exe),/NATIVE_ELF64_STATIC_EXEC_REQUIRED|NATIVE_ELF_DYNAMIC_LOADER_DENIED/);
    writeFileSync(exe,'NOT_ELF'.padEnd(64,'x'));
    assert.throws(()=>validateStaticElfArtifact(exe),/NATIVE_ELF64_STATIC_EXEC_REQUIRED/);
  }finally{rmSync(dir,{recursive:true,force:true});}
});
