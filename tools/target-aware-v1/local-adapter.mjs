// Trusted local compiler checkouts only. This runner is not a hostile-code sandbox.
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, rmSync, lstatSync, realpathSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, isAbsolute } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { compareValue, digest } from './validate.mjs';

const MAX = 16 * 1024 * 1024;
const env = { PATH: process.env.PATH || '/usr/bin:/bin', LANG: 'C', LC_ALL: 'C' };
const fail = message => { throw new Error(message); };
function regular(path, max = MAX) {
  const s = lstatSync(path);
  if (!s.isFile() || s.isSymbolicLink() || s.size < 1 || s.size > max ||
      realpathSync(path) !== path) fail('expected bounded canonical regular file: ' + path);
  const b = readFileSync(path);
  if (b.length !== s.size) fail('file changed during read');
  return b;
}
function invoke(bin, args, cwd) {
  const r = spawnSync(bin, args, {cwd, env, encoding:'utf8', timeout:30000, maxBuffer:65536, shell:false});
  if (r.error || r.signal || r.status === null) fail('process infrastructure failure: ' + (r.error?.message || r.signal));
  return r;
}
function git(root, args) {
  const r = invoke('git', ['-C',root,...args], root);
  if (r.status !== 0) fail('git verification failed');
  return r.stdout.trim();
}
function checkout(root, commit, tree) {
  if (!isAbsolute(root) || realpathSync(root) !== root ||
      !/^[a-f0-9]{40}$/.test(commit) || !/^[a-f0-9]{40}$/.test(tree) ||
      git(root,['rev-parse','--show-toplevel']) !== root ||
      git(root,['rev-parse','HEAD']) !== commit ||
      git(root,['rev-parse','HEAD^{tree}']) !== tree ||
      git(root,['status','--porcelain','--untracked-files=all']) !== '')
    fail('checkout pin or clean tree mismatch');
}
const gitBlob = bytes => createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex');
export function fixtureBytes(path, expectedBlob) {
  const b = regular(path, 262144);
  if (gitBlob(b) !== expectedBlob) fail('fixture blob mismatch');
  return b;
}
const numeric = JSON.parse(readFileSync(new URL('../../contracts/target-aware-v1/numeric-cases.json', import.meta.url),'utf8'));
const cases = new Map(numeric.cases.map(c => [c.id,Object.freeze({...c,expected:Object.freeze(c.expected)})]));
if (numeric.schema_version !== 1 || numeric.upstream_manifest_blob !== '8be4f89e994a4ace92f5de0f6d4b678ab60c1797' ||
    cases.size !== 11 || numeric.cases.length !== 11 ||
    numeric.cases.some(c => c.status !== 'NOT_EXECUTED' || !/^[a-f0-9]{40}$/.test(c.blob)))
  fail('invalid numeric case registry');
export function selectFixture(id = 'checked-i64-quotient', profile = 'strict-i64') {
  const c = cases.get(id);
  if (!c) fail('unknown numeric fixture');
  if (c.expected.kind !== 'i64' && profile !== 'js-program-v2') fail('structured failure oracle not implemented for this profile');
  compareValue(c.expected,c.expected);
  return c;
}
export const fixture = selectFixture();

// A separate bounded child executes the emitted artifact, not the source or a model.
const guest = `
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
const [lane,path,profile] = process.argv.slice(1);
let value, observedError=false;
if (lane === 'js-esm') {
  const m = await import(pathToFileURL(path).href);
  // Only a trusted, exact-pinned JS compiler's frozen error envelope is
  // a candidate observation. A generic host exception is not a match.
  try { value = await m.main(); }
  catch (e) {
    if (profile !== 'js-program-v2' || !(e instanceof RangeError) || !Object.isFrozen(e)) throw e;
    const field = k => Object.getOwnPropertyDescriptor(e,k)?.value;
    const code = field('code'), name = field('name'), abi = field('abiCode');
    if (!(code === 'division_by_zero' && name === 'DivisionByZeroError' && abi === 65538 ||
          code === 'integer_overflow' && name === 'ArithmeticOverflowError' && abi === 65537)) throw e;
    process.stdout.write(JSON.stringify({kind:'error',code})+'\\n');
    process.exitCode = 0;
    observedError = true;
  }
} else if (lane === 'rs-wasm') {
  const m = new WebAssembly.Module(readFileSync(path));
  if (WebAssembly.Module.imports(m).length !== 0) throw Error('unexpected host imports');
  const i = new WebAssembly.Instance(m, {});
  if (!(i.exports.panic_code instanceof WebAssembly.Global)) throw Error('missing panic ABI');
  value = i.exports.main();
  if (i.exports.panic_code.value !== 0) throw Error('guest panic');
} else throw Error('unsupported lane');
if (!observedError) {
  if (typeof value !== 'bigint' || value < -(1n<<63n) || value > (1n<<63n)-1n)
    throw Error('not a signed i64 BigInt');
  process.stdout.write(JSON.stringify({kind:'i64',value:value.toString()})+'\\n');
}
`;
export function decodeObservation(stdout) {
  const error = /^\{"kind":"error","code":"(division_by_zero|integer_overflow)"\}\n$/.exec(stdout);
  if (error && error[0] === stdout) return {kind:'error',code:error[1]};
  const m = /^\{"kind":"i64","value":"(0|-?[1-9][0-9]*)"\}\n$/.exec(stdout);
  if (!m || m[0] !== stdout) fail('invalid exact guest observation');
  const v = {kind:'i64',value:m[1]};
  compareValue(v,v); // check signed range
  return v;
}
export function execute(config) {
  const {lane, root, commit, tree, compiler_sha256, node_sha256, source} = config;
  if (lane !== 'js-esm' && lane !== 'rs-wasm') fail('lane adapter not implemented');
  const profile = config.profile ?? 'strict-i64';
  if (profile !== 'strict-i64' && !(lane === 'js-esm' && profile === 'js-program-v2'))
    fail('unsupported lane/profile combination');
  const fixture = selectFixture(config.case_id,profile);
  checkout(root,commit,tree);
  const node = realpathSync(process.execPath);
  if (digest(regular(node,128*1024*1024)) !== node_sha256) fail('Node binary digest mismatch');
  const compiler = lane === 'js-esm' ? join(root,'bin/oreslang-jsc.mjs') : resolve(config.attoc);
  if (digest(regular(compiler,128*1024*1024)) !== compiler_sha256) fail('compiler digest mismatch');
  const bytes = fixtureBytes(source,fixture.blob);
  const dir = mkdtempSync(join(tmpdir(),'atto-target-parity-'));
  try {
    const input = join(dir,'main.atto');
    const artifact = join(dir,lane === 'js-esm'?'guest.mjs':'guest.wasm');
    writeFileSync(input,bytes,{flag:'wx',mode:0o600});
    const c = lane === 'js-esm'
      ? invoke(node,[compiler,profile === 'js-program-v2'?'--program-v2':'--atto-scalar-v1',input,'--out='+artifact],dir)
      : invoke(compiler,[input,'-o',artifact],dir);
    if (c.status !== 0) {
      try { lstatSync(artifact); fail('failed compiler left an artifact'); } catch (e) { if (e.code !== 'ENOENT') throw e; }
      return {lane,case_id:fixture.id,status:'COMPILE_FAILED_UNCLASSIFIED',exit:c.status,stderr:c.stderr};
    }
    const before = digest(regular(artifact));
    const r = invoke(node,['--input-type=module','-e',guest,lane,artifact,profile],dir);
    const after = digest(regular(artifact));
    if (before !== after || digest(regular(input)) !== digest(bytes) ||
        digest(regular(compiler,128*1024*1024)) !== compiler_sha256 ||
        digest(regular(node,128*1024*1024)) !== node_sha256) fail('source, executable or artifact changed');
    checkout(root,commit,tree);
    if (r.status !== 0 || r.stderr !== '') return {lane,case_id:fixture.id,status:'GUEST_FAILED_UNCLASSIFIED',exit:r.status,stderr:r.stderr};
    const observed = decodeObservation(r.stdout);
    return {lane,profile,case_id:fixture.id,compiler_commit:commit,compiler_tree:tree,compiler_sha256,
      node_sha256,source_blob:fixture.blob,source_sha256:digest(bytes),artifact_sha256:before,
      artifact_before_run_sha256:before,artifact_after_run_sha256:after,observed,
      status:compareValue(fixture.expected,observed)?'LOCAL_CANDIDATE_MATCH':'LOCAL_CANDIDATE_MISMATCH',
      claim:'UNAUTHENTICATED_LOCAL_OBSERVATION_NOT_PARITY_PASS'};
  } finally { rmSync(dir,{recursive:true,force:true}); }
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    if (process.argv.length !== 3) fail('usage: node local-adapter.mjs config.json');
    const result = execute(JSON.parse(regular(resolve(process.argv[2]),65536)));
    console.log(JSON.stringify(result));
    // No release-gate success is minted by this local adapter.
    process.exitCode = result.status === 'LOCAL_CANDIDATE_MATCH' ? 0 : 1;
  } catch (e) {
    console.error('NOT_EXECUTED_OR_INFRASTRUCTURE_FAILURE: '+e.message);
    process.exitCode = 2;
  }
}
