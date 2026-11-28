import test from 'node:test';
import assert from 'node:assert/strict';
import {classifyJvmStderr as classify} from '../tools/comparator-jvm-stderr.mjs';
const home='/home/runner';
const w=[
 'WARNING: A terminally deprecated method in sun.misc.Unsafe has been called',
 'WARNING: sun.misc.Unsafe::objectFieldOffset has been called by com.oracle.truffle.api.nodes.NodeClassImpl$NodeFieldData (file:/home/runner/.m2/repository/org/graalvm/truffle/truffle-api/25.3.4.1/truffle-api-25.3.4.1.jar)',
 'WARNING: Please consider reporting this to the maintainers of class com.oracle.truffle.api.nodes.NodeClassImpl$NodeFieldData',
 'WARNING: sun.misc.Unsafe::objectFieldOffset will be removed in a future release'
].join('\n')+'\n';
test('exact warning retained as host diagnostic',()=>assert.deepEqual(classify(w,home),{hostWarning:w,guestStderr:''}));
test('unexpected guest stderr remains fatal to strict oracle',()=>assert.deepEqual(classify(w+'unexpected error\n',home),{hostWarning:w,guestStderr:'unexpected error\n'}));
test('no generic stripping and no altered-version stripping',()=>{
 for(const bad of ['WARNING: other\n',w.replace('25.3.4.1','25.3.4.2'),w+w,w.slice(0,-1),'prefix\n'+w]){
   const result=classify(bad,home);
   assert.notEqual(result.guestStderr,'');
 }
});
test('clean stderr and invalid home',()=>{
 assert.deepEqual(classify('',home),{hostWarning:null,guestStderr:''});
 assert.deepEqual(classify(w,'/home/runner\n'),{hostWarning:null,guestStderr:w});
});
