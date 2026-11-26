//! Deliberate v3-to-Rust registry profile adapter, not native v2 manifest
//! syntax or stdout parity. Verifies original pins and exact fixture grammar,
//! maps explicit namespaces to registry directories, retains both unnamespaced
//! sources in the lock, and encodes observed scalar calls in base 100.
#![cfg(all(feature="experimental-incremental-cache",target_os="linux",target_arch="x86_64"))]
use atto_compiler::{experimental_incremental::{HostSession,BuildIdentity,Snapshot,RegistryDependency,Mode,Outcome},
 package_resolver::SnapshotSource,registry_namespace::{FrozenRegistry,compile_registry_guest},library::TargetOs,sha256::{sha256,hex_digest}};
use std::{collections::BTreeMap,fs,io::Write,os::unix::fs::DirBuilderExt,path::PathBuf,process::{Command,Stdio}};
const MASTER:[u8;32]=[7;32];
const PKG:&str="@org/project/checkout";
const MANIFEST:&str="[package]\nname=\"@org/project/checkout\"\nversion=\"1.0.0\"\n[namespaces]\nbilling=\"src/billing/\"\nidentity=\"src/identity/\"\ntelemetry=\"src/telemetry/\"\n";
const ROOT:&str="[package]\nname=\"app\"\nversion=\"1.0.0\"\n";
const PIN:&str="8e5170a12e8950f934538ae7055d112a147361903f330b8632eaa71df5a2ebae";
const EDIT:&str="85f4245442b6854150a0e8247361287e69d6d355f1aeb8383f25373a0ab77f2e";
const ABI:&str="dbe6a14b22fba63b297c140b7b6b0f0e49e6a46b5646fc8d23682fd0c66ef069";
struct Temp(PathBuf);impl Drop for Temp{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn digest(files:&BTreeMap<String,String>)->String{
 let text=files.iter().map(|(p,s)|format!("{p}\n{}\n",hex_digest(s.as_bytes()))).collect::<String>();hex_digest(text.as_bytes())
}
fn adapted(files:&BTreeMap<String,String>)->Vec<(String,String)>{
 files.iter().map(|(p,s)|{
  let file=p.strip_prefix("src/").unwrap().strip_suffix(".ores").unwrap();
  if let Some(header)=s.strip_prefix("namespace "){
   let (ns,_)=header.split_once(';').unwrap();assert!(matches!(ns,"billing"|"identity"|"telemetry"));
   (format!("src/{ns}/{file}.atto"),format!("use {s}"))
  }else{assert!(file.starts_with("local"));(format!("src/local/{file}.atto"),s.clone())}
 }).collect()
}
fn inputs(files:&[(String,String)])->Vec<SnapshotSource<'_>>{files.iter().map(|(p,s)|SnapshotSource{path:p,source:s}).collect()}
fn lock(manifest:&str,inputs:&[SnapshotSource<'_>])->String{
 let mut s=format!("[lock]\nmanifest_sha256=\"{}\"\n[files]\n",hex_digest(manifest.as_bytes()));
 for i in inputs{s.push_str(&format!("\"{}\"=\"{}\"\n",i.path,hex_digest(i.source.as_bytes())));}s
}
fn execute(bytes:&[u8],expected:&str){
 let mut child=Command::new("node").args(["-e","const a=require('node:assert/strict'),b=require('node:fs').readFileSync(0);a.ok(WebAssembly.validate(b));const m=new WebAssembly.Module(b);a.deepEqual(WebAssembly.Module.imports(m),[]);const i=new WebAssembly.Instance(m,{});a.equal(i.exports.main(),BigInt(process.argv[1]));a.equal(i.exports.panic_code.value,0);",expected]).stdin(Stdio::piped()).spawn().unwrap();
 child.stdin.take().unwrap().write_all(bytes).unwrap();assert!(child.wait().unwrap().success());
}
#[test]fn shared_fixture_registry_profile(){
 let root=PathBuf::from(std::env::var("ATTO_V3_FIXTURE").unwrap());let mut files=BTreeMap::new();
 for entry in fs::read_dir(root.join("checkout/src")).unwrap(){let e=entry.unwrap();assert!(e.file_type().unwrap().is_file());files.insert(format!("src/{}",e.file_name().to_str().unwrap()),fs::read_to_string(e.path()).unwrap());}
 assert_eq!(files.len(),12);assert_eq!(digest(&files),PIN);
 let provider_manifest=fs::read_to_string(root.join("checkout/.ores.pkg.toml")).unwrap();
 assert_eq!(hex_digest(provider_manifest.as_bytes()),"72721b058d5367d74ed1bc4694e79ad13742db24513a192fb25851a3d391912f");
 let mut applications=Vec::new();
 for (client,ns,ids) in [("client","billing",0..4),("observer","identity",4..7)]{
  let original=fs::read_to_string(root.join(client).join("src/main.ores")).unwrap();
  let manifest=fs::read_to_string(root.join(client).join(".ores.pkg.toml")).unwrap();assert!(manifest.contains(PIN));
  let mut expected=format!("namespace app;\nimport namespace {ns} as provider from \"checkout\";\npub fnc main(): void {{\n");
  let mut expr=String::from("0");for i in ids{expected.push_str(&format!("  stdio.println(provider.function{i}());\n"));expr=format!("({expr})*100+provider.function{i}()");}
  expected.push_str("  return;\n}\n");assert_eq!(original,expected);
  applications.push(format!("import namespace {ns} as provider from pkg '{PKG}'; pub fnc main(): int {{ return {expr}; }}"));
 }
 let t=Temp(std::env::temp_dir().join(format!("atto-v3-registry-{}",std::process::id())));
 fs::DirBuilder::new().mode(0o700).create(&t.0).unwrap();let cache=t.0.join("cache");let authority=t.0.join("authority");
 for d in [&cache,&authority]{fs::DirBuilder::new().mode(0o700).create(d).unwrap();}
 let compiler=hex_digest(&fs::read("/proc/self/exe").unwrap());let toolchain=hex_digest(b"fixture Rust compiler and Node");
 let open=||HostSession::open(&cache,&authority,"tenant/v3",BuildIdentity{compiler_sha256:&compiler,toolchain_sha256:&toolchain,target:TargetOs::Posix}).unwrap();
 let mut session=open();let pin=sha256(&session.proposed_record(1,MASTER).unwrap());session.provision(1,MASTER,&pin).unwrap();
 let base=adapted(&files);let bi=inputs(&base);let bl=lock(MANIFEST,&bi);let bp=hex_digest(bl.as_bytes());
 let dep=||RegistryDependency{name:PKG,manifest:MANIFEST,lockfile:&bl,trusted_lock_sha256:&bp,sources:&bi};
 let roots=[SnapshotSource{path:"main.atto",source:&applications[0]}];let rl=lock(ROOT,&roots);let rp=hex_digest(rl.as_bytes());
 let snap=||Snapshot{entry:"main.atto",manifest:ROOT,lockfile:&rl,trusted_lock_sha256:&rp,sources:&roots};
 let obs=[SnapshotSource{path:"main.atto",source:&applications[1]}];let ol=lock(ROOT,&obs);let op=hex_digest(ol.as_bytes());
 let observer=||Snapshot{entry:"main.atto",manifest:ROOT,lockfile:&ol,trusted_lock_sha256:&op,sources:&obs};
 let cold=session.build_registry(Mode::Cold,&pin,MASTER,snap(),&[dep()]).unwrap();execute(&cold.bytes,"10203");
 let warm=session.build_registry(Mode::Incremental,&pin,MASTER,snap(),&[dep()]).unwrap();assert_eq!(warm.outcome,Outcome::Reused);assert_eq!(warm.bytes,cold.bytes);assert_eq!(warm.checked_namespaces,0);
 let identity=session.build_registry(Mode::Incremental,&pin,MASTER,observer(),&[dep()]).unwrap();execute(&identity.bytes,"40506");
 let old=files["src/unit0.ores"].clone();files.insert("src/unit0.ores".into(),old.replace("return 0","return 42"));assert_eq!(digest(&files),EDIT);
 let edited=adapted(&files);let ei=inputs(&edited);let el=lock(MANIFEST,&ei);let ep=hex_digest(el.as_bytes());
 assert_eq!(session.build_registry(Mode::Incremental,&pin,MASTER,snap(),&[RegistryDependency{sources:&ei,..dep()}]).unwrap_err().code,"E_LOCK_HASH");
 let next=||RegistryDependency{sources:&ei,lockfile:&el,trusted_lock_sha256:&ep,..dep()};
 let changed=session.build_registry(Mode::Incremental,&pin,MASTER,snap(),&[next()]).unwrap();execute(&changed.bytes,"42010203");
 let check=FrozenRegistry::verify(PKG,MANIFEST,&el,&ep,&ei).unwrap();assert_eq!(changed.bytes,compile_registry_guest(&applications[0],&[&check]).unwrap());
 let unchanged=session.build_registry(Mode::Incremental,&pin,MASTER,observer(),&[next()]).unwrap();execute(&unchanged.bytes,"40506");assert_eq!(unchanged.bytes,compile_registry_guest(&applications[1],&[&check]).unwrap());
 // Record unrelated artifact churn as a real parity gap, not a failure of guest output.
 let observer_artifact_changed=unchanged.bytes!=identity.bytes;
 println!("V3_RUST_OBSERVER_ARTIFACT_CHANGED={observer_artifact_changed}");
 assert!(!observer_artifact_changed,"unreachable namespace body must not change observer artifact");
 files.insert("src/unit0.ores".into(),"namespace billing;\npub fnc function0(int x): int { return x; }\n".into());assert_eq!(digest(&files),ABI);
 let bad=adapted(&files);let ai=inputs(&bad);let al=lock(MANIFEST,&ai);let ap=hex_digest(al.as_bytes());
 let bad_dep=RegistryDependency{sources:&ai,lockfile:&al,trusted_lock_sha256:&ap,..dep()};
 let rejected=session.build_registry(Mode::Incremental,&pin,MASTER,snap(),&[bad_dep]).unwrap_err();
 assert_eq!(rejected.code,"E_ARITY");
 println!("V3_RUST_ABI_REJECT|{}|{}",rejected.code,rejected.message);
 let recovered=session.build_registry(Mode::Incremental,&pin,MASTER,snap(),&[next()]).unwrap();execute(&recovered.bytes,"42010203");assert_eq!(recovered.bytes,changed.bytes);
 if let Ok(dir)=std::env::var("ATTO_V3_ARTIFACTS"){
  let dir=PathBuf::from(dir);fs::create_dir(&dir).unwrap();
  for (lane,bytes) in [("cold",&cold.bytes),("warm",&warm.bytes),("edit",&changed.bytes),("observer",&identity.bytes),("observer-edit",&unchanged.bytes)]{
   fs::write(dir.join(format!("{lane}.wasm")),bytes).unwrap();
  }
 }
 println!("V3_RUST_REGISTRY_ADAPTER|guest=0/1/2/3->42/1/2/3|observer=4/5/6|warm-reuse|stale-pin|abi-denied|rollback|cold-checks={}|edit-checks={}|observer-edit-checks={}",cold.checked_namespaces,changed.checked_namespaces,unchanged.checked_namespaces);
}
