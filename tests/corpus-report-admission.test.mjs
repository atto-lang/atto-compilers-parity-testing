import test from 'node:test';
import assert from 'node:assert/strict';
import {inspectCorpusReport as inspect} from '../tools/corpus-report-admission.mjs';
const digest='f'.repeat(64);
function base(){return {schema_version:2,executed:false,cases:2,
  backends:['java','llvm'],
  records:['java','llvm'].flatMap(backend=>['01','02'].map(case_id=>({
   suite:'legacy',case_id,backend,source_sha256:digest,status:'not_run',
   compile:null,run:null,actual_diagnostic_category:null
  }))),summary:{not_run:4}};}
test('full Cartesian corpus is classified not executed even when source audit passes',()=>{
 const x=inspect(base());
 assert.equal(x.records,4);assert.equal(x.notRun,4);
 assert.equal(x.guestExecutionAttested,false);
 assert.equal(x.classification,'CORPUS_NOT_EXECUTED');
});
test('missing or duplicate records are rejected, not silently credited',()=>{
 for(const mutate of [
  x=>x.records.pop(),x=>x.records[3]={...x.records[2]},
  x=>x.records[0].backend='other',x=>x.records[0].source_sha256='bad',
  x=>x.summary.not_run=3,x=>x.backends=['java','java']
 ]){const x=base();mutate(x);assert.throws(()=>inspect(x),/NATIVE_CORPUS_UNVERIFIED/);}
});
test('not-run records cannot contain fabricated compile/guest receipts',()=>{
 for(const mutate of [
  x=>x.records[0].compile={status:'completed',exit_code:0},
  x=>x.records[0].run={status:'completed',exit_code:42},
  x=>x.records[0].actual_diagnostic_category='removed_operator',
  x=>x.executed=true
 ]){const x=base();mutate(x);assert.throws(()=>inspect(x),/NATIVE_CORPUS_UNVERIFIED/);}
});

test('Forge is a required recognized backend and cannot receive a different case set',()=>{
 const x=base(); x.backends.push('forge'); x.records.push(...x.records.filter(r=>r.backend==='java').map(r=>({...r,backend:'forge'}))); x.summary.not_run=6;
 assert.equal(inspect(x).notRun,6);
 const other=structuredClone(x);other.records.at(-1).case_id='different';assert.throws(()=>inspect(other),/case inventory/);
 const drift=structuredClone(x);drift.records.at(-1).source_sha256='a'.repeat(64);assert.throws(()=>inspect(drift),/case source identity/);
});
