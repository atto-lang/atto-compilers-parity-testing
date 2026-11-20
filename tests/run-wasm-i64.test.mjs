import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, writeFileSync, rmSync, symlinkSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {runWasmI64} from '../tools/guest/run-wasm-i64.mjs';

const runner=fileURLToPath(new URL('../tools/guest/run-wasm-i64.mjs',import.meta.url));
// Source-free synthetic Wasm module: main(): i64 { 42 }.
const valid = Uint8Array.from([0,97,115,109,1,0,0,0,1,5,1,96,0,1,126,3,2,1,0,7,8,1,4,109,97,105,110,0,0,10,6,1,4,0,66,42,11]);
const invalidImport = Uint8Array.from([0,97,115,109,1,0,0,0,1,4,1,96,0,0,2,7,1,1,120,1,121,0,0]);

test('guest host accepts only a real instantiated import-free wasm main i64',()=>{
 const dir=mkdtempSync(join(tmpdir(),'atto-wasm-host-'));
 try{
  const file=join(dir,'test.wasm');writeFileSync(file,valid);
  assert.equal(runWasmI64(file),'42');
  const result=spawnSync(process.execPath,[runner,file],{encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);
  assert.equal(result.stdout,'42\n');
  assert.equal(result.stderr,'');
 }finally{rmSync(dir,{force:true,recursive:true});}
});
test('rejects missing, relative, invalid and empty artifacts',()=>{
 assert.throws(()=>runWasmI64('relative.wasm'),/WASM_INPUT_NOT_ABSOLUTE/);
 const dir=mkdtempSync(join(tmpdir(),'atto-wasm-host-'));
 try{
  const file=join(dir,'invalid.wasm');writeFileSync(file,Buffer.from('faked wasm'));
  assert.throws(()=>runWasmI64(file),/WASM_MODULE_INVALID/);
  writeFileSync(file,Buffer.from(''));
  assert.throws(()=>runWasmI64(file),/WASM_INPUT_NOT_REGULAR_BOUNDED/);
 }finally{rmSync(dir,{force:true,recursive:true});}
});
test('CLI refuses nonexistent artifacts and extra arguments',()=>{
 const missing=spawnSync(process.execPath,[runner,'/no/such/guest.wasm'],{encoding:'utf8'});
 assert.equal(missing.status,2);assert.match(missing.stderr,/WASM_RUNNER_ERROR/);
 const extra=spawnSync(process.execPath,[runner,'/no/such/guest.wasm','extra'],{encoding:'utf8'});
 assert.equal(extra.status,2);assert.match(extra.stderr,/WASM_USAGE_ARTIFACT_REQUIRED/);
});

test('rejects host imports, rather than granting Wasm ambient capabilities',()=>{
 const dir=mkdtempSync(join(tmpdir(),'atto-wasm-imports-'));
 try {
  const file=join(dir,'imports.wasm');writeFileSync(file,invalidImport);
  assert.throws(()=>runWasmI64(file),/WASM_IMPORTS_NOT_ALLOWED/);
 }finally{rmSync(dir,{force:true,recursive:true});}
});
test('rejects symlink artifacts before instantiation',()=>{
 const dir=mkdtempSync(join(tmpdir(),'atto-wasm-symlink-'));
 try{
  const file=join(dir,'guest.wasm');writeFileSync(file,valid);
  const link=join(dir,'alias.wasm');
  symlinkSync(file,link);
  assert.throws(()=>runWasmI64(link),/WASM_INPUT_NOT_REGULAR_BOUNDED/);
 }finally{rmSync(dir,{force:true,recursive:true});}
});

test('preserves signed i64 bit pattern and never fabricates an overflow diagnostic',()=>{
 const dir=mkdtempSync(join(tmpdir(),'atto-wasm-trap-'));
 try{
  const negative=join(dir,'negative.wasm');const bytes=Uint8Array.from(valid);bytes[35]=126;
  writeFileSync(negative,bytes);
  assert.equal(runWasmI64(negative),'-2');
  const trap=join(dir,'trap.wasm');
  writeFileSync(trap,Uint8Array.from([...valid.slice(0,29),10,5,1,3,0,0,11]));
  const result=spawnSync(process.execPath,[runner,trap],{encoding:'utf8'});
  assert.equal(result.status,2);
  assert.match(result.stderr,/WASM_GUEST_TRAP_UNCLASSIFIED/);
  assert.doesNotMatch(result.stderr,/INTEGER_OVERFLOW|E_CONST_OVERFLOW/);
  assert.equal(result.stdout,'');
 }finally{rmSync(dir,{force:true,recursive:true});}
});
