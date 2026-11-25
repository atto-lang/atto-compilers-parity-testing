// Local parity-runner output audit, not authenticated GitHub or native evidence.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
const fail=why=>{throw Error('NATIVE_CORPUS_UNVERIFIED: '+why);};
const backends=new Set(['llvm','java','wasm','dotnet','forge','js','cranelift']);
const digest=/^[a-f0-9]{64}$/;
export function inspectCorpusReport(report){
 if(!report||report.schema_version!==2||typeof report.executed!=='boolean'||
    !Number.isSafeInteger(report.cases)||report.cases<=0||
    !Array.isArray(report.backends)||report.backends.length===0||
    !Array.isArray(report.records)||!report.summary||typeof report.summary!=='object')
    fail('malformed corpus report envelope');
 if(new Set(report.backends).size!==report.backends.length||
    report.backends.some(x=>!backends.has(x)))fail('backend inventory identity mismatch');
 const expected=report.cases*report.backends.length;
 if(report.records.length!==expected)fail('missing corpus/backend Cartesian product');
 const pairSet=new Set();
 const caseDigests=new Map();
 const totals=new Map();
 let notRun=0,other=0;
 for(const r of report.records){
   if(!r||typeof r.suite!=='string'||!r.suite||
      typeof r.case_id!=='string'||!r.case_id||
      !report.backends.includes(r.backend)||
      !digest.test(r.source_sha256)||typeof r.status!=='string')
       fail('missing case/backend/source identity');
   const caseKey=r.suite+'\0'+r.case_id;
   if(caseDigests.has(caseKey)&&caseDigests.get(caseKey)!==r.source_sha256)fail('case source identity differs across backends');
   caseDigests.set(caseKey,r.source_sha256);
   const key=caseKey+'\0'+r.backend;
   if(pairSet.has(key))fail('duplicate case/backend receipt');pairSet.add(key);
   if(r.status==='not_run'){
     notRun++;
     if(r.compile!==null||r.run!==null||r.actual_diagnostic_category!==null)
       fail('not-run record contains fabricated execution evidence');
   }else{
     other++;
     if(report.executed!==true)fail('unexecuted run contains executed case');
   }
   totals.set(r.status,(totals.get(r.status)||0)+1);
 }
 if(notRun>0&&report.executed===true&&notRun===expected)
   fail('empty execute request cannot establish native execution');
 if(caseDigests.size!==report.cases)fail('case inventory differs across backends');
 const reported=Object.entries(report.summary);
 if(reported.length!==totals.size||
    reported.some(([kind,count])=>!Number.isSafeInteger(count)||count!==totals.get(kind)))
    fail('summary disagrees with independent per-record count');
 if(report.executed===false&&other>0)fail('source-only report claims execution');
 return Object.freeze({classification:notRun===expected?'CORPUS_NOT_EXECUTED':
    'CORPUS_OUTPUT_REVIEW_REQUIRED',cases:report.cases,backends:report.backends.length,
    records:expected,notRun,guestExecutionAttested:false});
}
if(process.argv[1]&&import.meta.url===new URL('file://'+resolve(process.argv[1])).href){
 try{
   if(process.argv.length!==3)fail('usage: node tools/corpus-report-admission.mjs report.json');
   const result=inspectCorpusReport(JSON.parse(readFileSync(process.argv[2],'utf8')));
   if(result.classification!=='CORPUS_NOT_EXECUTED')fail('manual guest attribution required');
   console.log(JSON.stringify(result));
 }catch(e){console.error(e.message);process.exitCode=1;}
}
