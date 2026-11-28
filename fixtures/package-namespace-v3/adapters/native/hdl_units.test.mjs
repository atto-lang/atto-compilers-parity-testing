// The common prepare.mjs verifies the original pins and preserves return
// expressions. This adapter changes only the scalar type to unsigned u8.
// The simulator receives no external input values.
import test from 'node:test';
import assert from 'node:assert/strict';
import {compileSource,IncrementalSession,evaluateReference} from '../src/compiler.mjs';
import {readFileSync,writeFileSync,mkdtempSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync} from 'node:child_process';
const root=process.env.ATTO_NATIVE_INPUT;
function simulate(code,expected){
 const dir=mkdtempSync(join(tmpdir(),'atto-v3-hdl-'));
 try{const rtl=join(dir,'dut.sv'),tb=join(dir,'tb.sv'),exe=join(dir,'sim');writeFileSync(rtl,code);
 writeFileSync(tb,"module tb; wire [7:0] result; ores_main dut(.out_result(result)); initial begin #1; if(result !== 8'd"+expected+") $fatal(1); $display(\"V3_RTL_OK\"); $finish; end endmodule");
 execFileSync('iverilog',['-g2012','-s','tb','-o',exe,rtl,tb]);assert.match(execFileSync('vvp',[exe],{encoding:'utf8'}),/V3_RTL_OK/);
 }finally{rmSync(dir,{recursive:true,force:true});}
}
test('authenticated v3 standalone provider literals, real RTL simulation',()=>{
 const sessions=Array.from({length:7},()=>new IncrementalSession()),prior=[];
 const source=(lane,i)=>{const text=readFileSync(join(root,lane,'unit'+i+'.atto'),'utf8');assert.match(text,/^pub fnc main\(\): int \{ return [0-9]+; \}\n$/);return text.replace(': int',': u8');};
 for(const lane of ['cold','warm','edit']){
  let hits=0;
  for(let i=0;i<7;i++){
   const s=source(lane==='warm'?'cold':lane,i),result=sessions[i].build(s,{cold:lane==='cold'});
   hits+=Number(result.reused);assert.equal(result.systemVerilog,compileSource(s).systemVerilog);
   if(lane==='cold')prior.push(result.systemVerilog);else if(lane==='warm'||i!==0)assert.equal(result.systemVerilog,prior[i]);
   const expected=lane==='edit'&&i===0?42:i;
   assert.equal(evaluateReference(result.program,{}),BigInt(expected));simulate(result.systemVerilog,expected);
   console.log('V3_HDL_GUEST|'+lane+'|'+i+'|'+expected);
  }
  assert.equal(hits,lane==='cold'?0:lane==='warm'?7:6);console.log('V3_HDL_UNITS|'+lane+'|hits='+hits);
 }
 assert.throws(()=>sessions[0].build('pub fnc main(): u8 { return 256; }'),e=>e.code==='HDL_LITERAL_RANGE');
 assert.equal(sessions[0].build(source('edit',0)).reused,true);
 console.log('V3_HDL_ROLLBACK|range-denied|zero-input|adapted-not-namespace-linking');
});
