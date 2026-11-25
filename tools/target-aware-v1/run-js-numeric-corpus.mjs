// Trusted CI checkouts only. Do not treat a local match as a release gate.
import {readFileSync,realpathSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {execFileSync} from 'node:child_process';
import {execute,fixtureBytes} from './local-adapter.mjs';
import {digest} from './validate.mjs';

const root=realpathSync(resolve('../compiler'));
const corpus=realpathSync(resolve('../corpus'));
const manifest=JSON.parse(readFileSync(new URL('../../contracts/target-aware-v1/numeric-cases.json',import.meta.url),'utf8'));
const pin={
  commit:'472087eb1900135bd469ff86a8e5c3a39202fbf3',
  tree:'4bc6ea66cf791eaf6d5b7383743a81bbac8fe3f6',
  corpus:'57293adb917ef776795b78178cf66ceb09e9bb80',
  manifest:'8be4f89e994a4ace92f5de0f6d4b678ab60c1797'
};
const git=(path,...args)=>execFileSync('git',['-C',path,...args],{encoding:'utf8'}).trim();
if(git(root,'rev-parse','HEAD')!==pin.commit||git(root,'rev-parse','HEAD^{tree}')!==pin.tree||
   git(corpus,'rev-parse','HEAD')!==pin.corpus)throw Error('incorrect compiler or corpus checkout');
fixtureBytes(join(corpus,'fixtures/numeric-guest-v1/manifest.json'),pin.manifest);
const compiler_sha256=digest(readFileSync(join(root,'bin/oreslang-jsc.mjs')));
const node_sha256=digest(readFileSync(realpathSync(process.execPath)));
let matched=0;
for(const c of manifest.cases){
  if(!/^[a-z0-9-]+\.atto$/.test(c.path))throw Error('invalid fixture path');
  try{
    const result=execute({lane:'js-esm',profile:'js-program-v2',case_id:c.id,root,commit:pin.commit,tree:pin.tree,
      compiler_sha256,node_sha256,source:join(corpus,'fixtures/numeric-guest-v1',c.path)});
    console.log(JSON.stringify(result));
    if(result.status==='LOCAL_CANDIDATE_MATCH')matched++;
  }catch(error){
    console.log(JSON.stringify({case_id:c.id,status:'NOT_EXECUTED_OR_INFRASTRUCTURE_FAILURE',diagnostic:error.message}));
  }
}
console.log(JSON.stringify({claim:'TRUSTED_CI_CANDIDATE_OBSERVATIONS_NOT_PORTABLE_CERTIFICATION',matched,total:manifest.cases.length}));
if(manifest.cases.length!==11||matched!==11)process.exitCode=1;
