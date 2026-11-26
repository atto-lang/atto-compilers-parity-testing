// External V3 conformance adapter, not native package grammar/linker.
// Authenticating host compiles original provider source through checked RTL compiler,
// then composes independent RTL provider and argument modules.
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const {compileCheckedNamespaceProvider}=await import(pathToFileURL(process.env.ATTO_HDL_COMPILER).href);
function compile(source,ns,symbol,arity,prefix='provider_'){
 return compileCheckedNamespaceProvider(source,ns,symbol,arity,prefix.slice(0,-1)).systemVerilog;
}
function link(calls,exports,argumentsByName){
 const unique=[...new Set(calls.map(c=>c.symbol))];
 const rtl=unique.map(s=>exports.get(s).code).concat([...argumentsByName.values()].map(x=>x.code)).join('\n');
 const wires=[...argumentsByName.keys()].map(s=>'wire signed [63:0] a_'+s+'; wire f_'+s+'; ores_argument_'+s+' arg_'+s+'(a_'+s+',f_'+s+');');
 for(const [i,c] of calls.entries()){
  wires.push('wire signed [63:0] v_'+i+'; wire fault_'+i+'; ores_provider_'+c.symbol+' dut_'+i+'('+[...c.args.map(a=>'a_'+a),'v_'+i,'fault_'+i].join(',')+');');
 }
 const faults=[...argumentsByName.keys()].map(s=>'f_'+s).concat(calls.map((_,i)=>'fault_'+i));
 const tb='module tb;\n'+wires.join('\n')+'\ninitial begin #1;\nif (('+faults.join(' | ')+') !== 1\'b0) $fatal(1,"checked arithmetic fault");\n'+calls.map((_,i)=>'$display("%0d",v_'+i+');').join('\n')+'\n$finish;\nend\nendmodule\n';
 return rtl+'\n'+tb;
}

const deny=(s)=>{throw Error('E_NAMESPACE_PROFILE: '+s)};
const sha=s=>createHash('sha256').update(s,'utf8').digest('hex');
const name='[A-Za-z_][A-Za-z0-9_]{0,127}';
const namespace=name+'(?:\\.'+name+')*';
const header='schema_version = "2"\\n\\[project\\]\\nroot = "\\."\\n';
const pathRe=/^src\/[A-Za-z0-9_-]{1,120}\.(?:ores|atto)$/;
function bounded(s,limit){
  if(typeof s!=='string'||s.length>limit||!s.isWellFormed()||Buffer.byteLength(s)>limit||s.includes('\0'))deny('source budget');
  return s;
}
function exactMatch(re,s){const m=re.exec(s);if(!m||m[0]!==s)deny('unsupported syntax');return m;}
function providerManifest(s){
  bounded(s,65536);
  const m=exactMatch(new RegExp('^'+header+'\\[packages\\.('+name+')\\]\\nfiles = \\[([\\s\\S]*)\\]\\n$'),s);
  const raw=m[2].trim().replace(/,$/,'');
  const files=raw.split(',').map(x=>exactMatch(/^"([^"]+)"$/,x.trim())[1]);
  if(files.length<1||files.length>128||new Set(files).size!==files.length||files.some(x=>!pathRe.test(x)))deny('inventory paths');
  return {packageName:m[1],files:files.sort()};
}
function consumerManifest(s){
  bounded(s,65536);
  const m=exactMatch(new RegExp('^'+header+'\\[packages\\.('+name+')\\]\\nfiles = \\["src/main\\.ores"\\]\\n\\[namespace_dependencies\\.('+name+')\\]\\nmanifest = "\\.\\./('+name+')/\\.ores\\.pkg\\.toml"\\npackage = "('+name+')"\\nmanifest_sha256 = "([a-f0-9]{64})"\\nsources_sha256 = "([a-f0-9]{64})"\\n$'),s);
  if(m[3]!==m[4])deny('manifest locator');
  return {consumer:m[1],alias:m[2],packageName:m[4],manifestPin:m[5],sourcesPin:m[6]};
}
function classify(source){
  const m=exactMatch(new RegExp('^(?:namespace ('+namespace+');\\n)?(pub fnc ('+name+')\\(([^\\n()]*)\\): int \\{ return ([^\\n{};]+); \\})\\n$'),source);
  const params=m[4]===''?[]:m[4].split(', ');
  if(params.length>4)throw Error('E_NAMESPACE_ARITY: runtime parameter budget');
  if(params.some(p=>!new RegExp('^int '+name+'$').test(p))||new Set(params).size!==params.length)deny('exact integer parameters required');
  if(!/^[A-Za-z_0-9+*() -]{1,1024}$/.test(m[5]))deny('closed integer expression required');
  return {namespace:m[1]??null,declaration:m[2],symbol:m[3],arity:params.length};
}
function consumerSource(source){
  bounded(source,65536);
  const m=exactMatch(new RegExp('^namespace '+namespace+';\\nimport namespace ('+namespace+') as ('+name+') from "('+name+')";\\npub fnc main\\(\\): void \\{\\n([\\s\\S]*)  return;\\n\\}\\n$'),source);
  const lines=m[4].split('\n');
  if(lines.pop()!==''||!lines.length||lines.length>128)deny('consumer calls');
  const calls=lines.map(line=>{
    const x=exactMatch(new RegExp('^  stdio\\.println\\(('+name+')\\.('+name+')\\(([^;{}\\n]*)\\)\\);$'),line);
    if(x[1]!==m[2])deny('foreign alias');
    const args=x[3]===''?[]:x[3].split(',');
    if(args.length>4||args.some(a=>!a.trim()||a.length>1024||!/^[0-9+*() -]+$/.test(a)))deny('closed integer argument required');
    return {symbol:x[2],args};
  });
  return {namespace:m[1],alias:m[3],calls};
}


export class CheckedHdlV3Adapter {
  #prior=new Map();#domain=null;#linked=null;#linkKey=null;#args=new Map();
  build({consumer,consumerManifest:cm,providerManifest:pm,sources}={}){
    const dep=consumerManifest(cm),pkg=providerManifest(pm),selector=consumerSource(consumer);
    if(dep.alias!==selector.alias||dep.packageName!==pkg.packageName||sha(pm)!==dep.manifestPin)deny('manifest pin or identity mismatch');
    if(!(sources instanceof Map)||sources.size!==pkg.files.length)deny('inventory size');
    const snapshot=new Map();let total=0;
    for(const p of pkg.files){
      const source=bounded(sources.get(p),65536);total+=Buffer.byteLength(source);
      if(total>4*1024*1024)deny('inventory budget');
      snapshot.set(p,source);
    }
    const inventory=[...snapshot].map(([p,s])=>p+'\n'+sha(s)+'\n').join('');
    if(sha(inventory)!==dep.sourcesPin)deny('source pin mismatch');
    // Manifest identity excludes source body pin so a repinned body edit
    // retains domain. Include consumer identity and namespace selection.
    const domain=sha(JSON.stringify([dep.consumer,dep.alias,dep.manifestPin,selector.namespace]));
    const prior=domain===this.#domain?this.#prior:new Map(),next=new Map(),exports=new Map();
    let classified=0,classificationReused=0,compiled=0,artifactReused=0;
    for(const [p,source] of snapshot){
      let entry=prior.get(p);
      if(entry?.source===source)classificationReused++;
      else {entry={source,info:classify(source),code:null};classified++;}
      if(entry.info.namespace===selector.namespace){
        if(exports.has(entry.info.symbol))deny('duplicate selected symbol');
        // Actual Atto scalar compiler, not a host evaluator. Stage output
        // before updating any memo; failed validation cannot publish it.
        if(entry.code===null){entry={...entry,code:compile(source,entry.info.namespace,entry.info.symbol,entry.info.arity)};compiled++;}
        else artifactReused++;
        exports.set(entry.info.symbol,entry);
      }
      next.set(p,entry);
    }
    let argumentCompiled=0,argumentReused=0;
    const nextArgs=new Map(),calls=[];
    for(const {symbol,args} of selector.calls){
      if(!exports.has(symbol))throw Error('E_NAMESPACE_SYMBOL: unavailable selected public symbol '+symbol);
      if(args.length!==exports.get(symbol).info.arity)throw Error('E_NAMESPACE_ARITY: incompatible selected call');
      const names=args.map(expr=>{
        const name='arg'+nextArgs.size;
        let entry=domain===this.#domain?this.#args.get(name):null;
        if(entry?.expr===expr)argumentReused++;
        else{
          entry={expr,code:compile('namespace '+selector.namespace+';\npub fnc '+name+'(): int { return '+expr+'; }\n',selector.namespace,name,0,'argument_')};
          argumentCompiled++;
        }
        nextArgs.set(name,entry);return name;
      });
      calls.push({symbol,args:names});
    }
    const key=sha(JSON.stringify([calls,calls.map(c=>[c.symbol,exports.get(c.symbol).code]),[...nextArgs]]));
    const hit=domain===this.#domain&&key===this.#linkKey;
    const code=hit?this.#linked:link(calls,exports,nextArgs);
    const relinked=hit?0:1;
    this.#domain=domain;this.#prior=next;this.#linked=code;this.#linkKey=key;this.#args=nextArgs;
    return {code,classified,classificationReused,compiled,artifactReused,relinked,domain,argumentCompiled,argumentReused};
  }
}
