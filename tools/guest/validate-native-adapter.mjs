// Strict Linux/x86-64 Cranelift object -> static ELF -> bubblewrap admission.
// This guards local observations, NOT independently attested compiler parity.
import {lstatSync, realpathSync, readFileSync} from 'node:fs';
import {basename} from 'node:path';
import {canonicalI64} from '../java-rust-observation-policy.mjs';

const HEX64=/^[0-9a-f]{64}$/;
const same=(a,b)=>Array.isArray(a)&&a.length===b.length&&a.every((v,i)=>v===b[i]);
const LIMITS=['--as=536870912','--cpu=5','--nproc=32',
  '--nofile=64','--fsize=1048576','--core=0','--'];
const SANDBOX_FLAGS=['--unshare-all','--die-with-parent','--new-session','--clearenv',
  '--tmpfs','/','--ro-bind','{artifact}','/guest','--dev','/dev',
  '--proc','/proc','--','/guest'];

export function validateNativeAdapterShape(a) {
  if(!a||a.backend!=='rust-cranelift')throw new Error('NATIVE_BACKEND_MISMATCH');
  if(process.platform!=='linux'||process.arch!=='x64')throw new Error('NATIVE_REQUIRES_LINUX_X86_64');
  if(!same(a.compile_argv,[a.compiler_executable,'--strict-aot','--checked-i64','{source}','{object}']))
    throw new Error('NATIVE_COMPILER_ARGV_UNTRUSTED');
  if(!HEX64.test(a.linker_sha256||'')||!same(a.link_argv,[a.linker_executable,'-static','{object}','-o','{artifact}']))
    throw new Error('NATIVE_LINKER_ARGV_UNTRUSTED');
  if(basename(a.guest_runner_executable||'')!=='prlimit'||
      basename(a.native_sandbox_executable||'')!=='bwrap'||
      !HEX64.test(a.native_sandbox_sha256||'')||
      !same(a.run_argv,[a.guest_runner_executable,...LIMITS,a.native_sandbox_executable,...SANDBOX_FLAGS])||
      a.value_mode!=='exit_code')
    throw new Error('NATIVE_SANDBOX_ARGV_UNTRUSTED');
  if(a.native_guest_driver!==undefined||a.native_host_wrapper!==undefined)
    throw new Error('NATIVE_UNAPPROVED_DRIVER');
  return {linker_executable:a.linker_executable,linker_sha256:a.linker_sha256,
    sandbox_executable:a.native_sandbox_executable,sandbox_sha256:a.native_sandbox_sha256};
}

export function supportsNativeOracle(fixture) {
  if(fixture?.kind==='reject'||fixture?.kind==='observe')return true;
  if(fixture?.kind!=='value')return false;
  const i64=canonicalI64(fixture.expected_value);
  return i64!==null&&i64>=0n&&i64<=125n;
}

// Reject forged dynamic/shared objects before any native execution. Static
// Linux x86-64 ELF ET_EXEC with at least one PT_LOAD and no dynamic loader.
export function validateStaticElfArtifact(path) {
  const st=lstatSync(path);
  if(!st.isFile()||st.isSymbolicLink()||realpathSync(path)!==path||
      st.size<64||st.size>128*1024*1024)throw new Error('NATIVE_ARTIFACT_NOT_BOUNDED_REGULAR');
  const data=readFileSync(path);
  if(data.toString('binary',0,4)!=='\x7fELF'||data[4]!==2||data[5]!==1||
      data[6]!==1||data.readUInt16LE(16)!==2||data.readUInt16LE(18)!==62)
    throw new Error('NATIVE_ELF64_STATIC_EXEC_REQUIRED');
  const phoffBig=data.readBigUInt64LE(32);
  const phentsize=data.readUInt16LE(54), phnum=data.readUInt16LE(56);
  if(phoffBig>BigInt(Number.MAX_SAFE_INTEGER)||phentsize<56||phnum<1||phnum>512)
    throw new Error('NATIVE_ELF_PROGRAM_HEADERS_INVALID');
  const phoff=Number(phoffBig);
  if(phoff<64||phoff+phentsize*phnum>data.length)
    throw new Error('NATIVE_ELF_PROGRAM_HEADERS_INVALID');
  let loadCount=0;
  for(let i=0;i<phnum;i++){
    const type=data.readUInt32LE(phoff+i*phentsize);
    if(type===1)loadCount++;
    if(type===2||type===3)throw new Error('NATIVE_ELF_DYNAMIC_LOADER_DENIED');
  }
  if(loadCount===0)throw new Error('NATIVE_ELF_NO_LOAD_SEGMENTS');
  return true;
}
