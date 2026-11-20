import test from 'node:test';
import assert from 'node:assert/strict';
import {BACKENDS,validateConfig,parseSuiteManifest}
  from '../tools/run-stdio-conformance.mjs';

function config() {
  return {
    backend:'rs',toolchain:'Rust 1.90 (explicitly supplied, not detected)',
    compilerRepo:{path:'/tmp/compiler-source',sha:'a'.repeat(40)},
    suiteRepo:{path:'/tmp/conformance-source',sha:'b'.repeat(40)},
    compile:['/usr/local/bin/attoc','{source}','-o','{artifact}'],
    execute:['/usr/local/bin/atto-run','{artifact}'],
    timeoutMs:10000,
  };
}
function manifest() {
  const ids=['checked_echo','checked_echo_readch_alias',
    'checked_echo_is_variants','checked_echo_is_variants_readch',
    'stdio_is_pattern_fallback'];
  return {
    schema:'atto.stdio.channel.endpoints.v1',
    status:'TARGET_NOT_EXECUTED',
    backend_status:Object.fromEntries(BACKENDS.map(name=>[name,'NOT_EXECUTED'])),
    required_cases:[
      ...ids.map(id=>({id,kind:id==='stdio_is_pattern_fallback'?
        'source-compile-pass':'guest-pass'})),
      ...Array.from({length:13},(_,i)=>({
        id:'negative_'+i,kind:'compile-fail'
      })),
    ],
  };
}
test('requires explicit compiler/guest commands and pinned source trees',()=>{
  assert.equal(validateConfig(config()).backend,'rs');
  for(const alter of [
    {backend:'unknown'},{compilerRepo:{path:'relative',sha:'a'.repeat(40)}},
    {suiteRepo:{path:'/tmp/suite',sha:'bad'}},
    {execute:[]},{compile:['/usr/bin/true']},{compile:[]},
    {timeoutMs:999},
  ])assert.throws(()=>validateConfig({...config(),...alter}));
  const hdl={...config(),backend:'hdl',execute:[]};
  assert.equal(validateConfig(hdl).backend,'hdl');
});
test('refuses forged PASSED statuses, missing backends and missing guest cases',()=>{
  assert.equal(parseSuiteManifest(JSON.stringify(manifest())).required_cases.length,18);
  for(const transform of [
    s=>{s.backend_status.rs='PASS'},
    s=>{delete s.backend_status.llvm},
    s=>{s.required_cases=s.required_cases.filter(x=>x.id!=='checked_echo_is_variants')},
    s=>{s.required_cases.push(s.required_cases[0])},
    s=>{s.required_cases[0].kind='host-model'},
    s=>{s.status='PASS'},
  ]) {
    const copy=structuredClone(manifest());transform(copy);
    assert.throws(()=>parseSuiteManifest(JSON.stringify(copy)));
  }
});
test('separate HDL unsupported profile is not a fabricated stdin guest',()=>{
  const c=config();c.backend='hdl';delete c.execute;
  assert.equal(validateConfig(c).backend,'hdl');
  assert.ok(BACKENDS.includes('hdl'));
});
