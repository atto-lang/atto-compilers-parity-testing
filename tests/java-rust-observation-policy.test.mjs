import {test} from 'node:test';
import assert from 'node:assert/strict';
import {hasDiagnostic, classifyCompileNegative, classifyUnsupported, classifyGuest, assertObservationReport} from '../tools/java-rust-observation-policy.mjs';
const exit = (code, stderr='', stdout='') => ({exit_code:code, signal:null,error:null,stderr,stdout});
test('diagnostic codes must be exact category tokens, not substrings',()=>{
 assert.equal(hasDiagnostic('E_CONST_OVERFLOW: arithmetic failure','E_CONST_OVERFLOW'),true);
 assert.equal(hasDiagnostic('NOT_E_CONST_OVERFLOW_EXTRA','E_CONST_OVERFLOW'),false);
 assert.equal(hasDiagnostic('syntax error','E_CONST_OVERFLOW'),false);
 assert.equal(hasDiagnostic('E_CONST_OVERFLOW','E_CONST_OVERFLOW'),true);
 assert.equal(hasDiagnostic('E_CONST_OVERFLOW','../../bad'),false);
});
test('negative result requires nonzero compile exit, targeted code and no artifacts',()=>{
 assert.equal(classifyCompileNegative(exit(1,'E_CONST_OVERFLOW'),true,'E_CONST_OVERFLOW'),'OBSERVED_UNVERIFIED');
 assert.equal(classifyCompileNegative(exit(0,'E_CONST_OVERFLOW'),true,'E_CONST_OVERFLOW'),'MISMATCH');
 assert.equal(classifyCompileNegative(exit(1,'E_CONST_OVERFLOW'),false,'E_CONST_OVERFLOW'),'MISMATCH');
 assert.equal(classifyCompileNegative(exit(1,'syntax error'),true,'E_CONST_OVERFLOW'),'MISMATCH');
 assert.equal(classifyCompileNegative({...exit(null),signal:'SIGILL'},true,'E_CONST_OVERFLOW'),'INFRASTRUCTURE_FAILURE');
});
test('unsupported cannot be supplied by adapter without compiler evidence',()=>{
 assert.equal(classifyUnsupported(exit(1,'E_UNSUPPORTED_ACTOR'),true,'E_UNSUPPORTED_ACTOR'),'UNSUPPORTED');
 assert.equal(classifyUnsupported(exit(0,'E_UNSUPPORTED_ACTOR'),true,'E_UNSUPPORTED_ACTOR'),'MISMATCH');
 assert.equal(classifyUnsupported(exit(1,'syntax error'),true,'E_UNSUPPORTED_ACTOR'),'MISMATCH');
 assert.equal(classifyUnsupported(exit(1,'E_UNSUPPORTED_ACTOR'),false,'E_UNSUPPORTED_ACTOR'),'MISMATCH');
});
test('stdout integer matching requires exit zero and full exact i64',()=>{
 const t={kind:'value', expected_value:42};
 assert.equal(classifyGuest(t,exit(0,'','42\n'),'stdout_i64'),'OBSERVED_UNVERIFIED');
 assert.equal(classifyGuest(t,exit(1,'','42\n'),'stdout_i64'),'MISMATCH');
 assert.equal(classifyGuest(t,exit(0,'','42\nextra'),'stdout_i64'),'MISMATCH');
 assert.equal(classifyGuest(t,exit(42,'',''),'exit_code'),'OBSERVED_UNVERIFIED');
 assert.equal(classifyGuest({...t, expected_value:9223372036854776000},exit(0,'','9223372036854776000'),'stdout_i64'),'MISMATCH');
});
test('signals are not silently interpreted as overflow semantics',()=>{
 const t={kind:'trap',expected_diagnostic:'INTEGER_OVERFLOW'};
 assert.equal(classifyGuest(t,exit(2,'INTEGER_OVERFLOW'),'stdout_i64'),'OBSERVED_UNVERIFIED');
 assert.equal(classifyGuest(t,exit(2,'panic: unrelated'),'stdout_i64'),'MISMATCH');
 assert.equal(classifyGuest(t,{...exit(null),signal:'SIGILL'},'stdout_i64'),'INFRASTRUCTURE_FAILURE');
});
test('three undecided cases remain observations; nobody can fabricate PASS',()=>{
 assert.equal(classifyGuest({kind:'observe'},exit(0,'','hello'),'stdout_i64'),'OBSERVATION_ONLY');
 assert.throws(()=>assertObservationReport({guest_parity_certified:true,records:[]}));
 assert.throws(()=>assertObservationReport({guest_parity_certified:false,records:[{status:'PASSED'}]}));
 assert.throws(()=>assertObservationReport({guest_parity_certified:false,records:[{status:'UNSUPPORTED'}]}));
 assertObservationReport({guest_parity_certified:false,records:[{status:'UNSUPPORTED',compile:exit(1,'E_UNSUPPORTED')} ]});
});
