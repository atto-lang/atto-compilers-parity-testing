// Included as a test module of the real compiler binary. No source production
// modifications other than adding this test module.
use super::*;
use std::{fs,path::PathBuf,process::Command};
#[test]
fn shared_v3_native_units(){
 let root=PathBuf::from(std::env::var("ATTO_NATIVE_INPUT").unwrap());
 let target=std::env::var("ATTO_NATIVE_BACKEND").unwrap();
 let dir=root.join("run");fs::create_dir(&dir).unwrap();
 #[cfg(atto_v3_llvm)] let mut sessions:Vec<_>=(0..7).map(|_|IncrementalSession::default()).collect();
 #[cfg(not(atto_v3_llvm))] let mut sessions:Vec<_>=(0..7).map(|_|IncrementalObjectSession::default()).collect();
 let mut prior=Vec::new();
 for lane in ["cold","warm","edit"]{
  let mut hits=0;
  for i in 0..7{
   let actual=if lane=="warm"{"cold"}else{lane};
   let prefix=if target=="llvm"{"call"}else{"unit"};
   let source=fs::read_to_string(root.join(actual).join(format!("{prefix}{i}.atto"))).unwrap();
   #[cfg(atto_v3_llvm)] let (ir,hit)=sessions[i].build(&source,LibraryProfile::FreestandingCore,lane=="cold").unwrap();
   #[cfg(atto_v3_llvm)] let bytes={assert_eq!(ir,compile_ir_with_profile(&source,LibraryProfile::FreestandingCore).unwrap());ir.into_bytes()};
   #[cfg(not(atto_v3_llvm))] let (bytes,hit)=sessions[i].build(&source,lane=="cold").unwrap();
   #[cfg(not(atto_v3_llvm))] assert_eq!(bytes,backend::emit_object(&frontend::parse(&source).unwrap()).unwrap());
   if hit{hits+=1;}
   if lane=="cold"{prior.push(bytes.clone());}else if lane=="warm"||i!=0{assert_eq!(bytes,prior[i]);}
   let exe=dir.join("guest");
   #[cfg(atto_v3_llvm)] {let ir=dir.join("guest.ll");fs::write(&ir,&bytes).unwrap();assert!(Command::new("clang").arg(&ir).arg("-o").arg(&exe).status().unwrap().success());}
   #[cfg(not(atto_v3_llvm))] {let obj=dir.join("guest.o");let host=dir.join("host.c");fs::write(&obj,&bytes).unwrap();fs::write(&host,"#include <stdint.h>\nextern int32_t oreslang_scalar_entry(int64_t,int64_t,int64_t,int64_t,int64_t*);\nint main(void){int64_t out=-1;int32_t rc=oreslang_scalar_entry(0,0,0,0,&out);return rc==0?(int)out:255;}\n").unwrap();assert!(Command::new("cc").arg(&host).arg(&obj).arg("-o").arg(&exe).status().unwrap().success());}
   let expected=if lane=="edit"&&i==0{42}else{i as i32};assert_eq!(Command::new(&exe).status().unwrap().code(),Some(expected));
  }
  assert_eq!(hits,match lane{"cold"=>0,"warm"=>7,_=>6});
  println!("V3_NATIVE_UNITS|{target}|{lane}|hits={hits}|7-guests");
 }
 #[cfg(atto_v3_llvm)] {
   let bad=fs::read_to_string(root.join("abi/call0.atto")).unwrap();
   assert!(sessions[0].build(&bad,LibraryProfile::FreestandingCore,false).is_err());
   let good=fs::read_to_string(root.join("edit/call0.atto")).unwrap();assert!(sessions[0].build(&good,LibraryProfile::FreestandingCore,false).unwrap().1);
 }
 #[cfg(not(atto_v3_llvm))] {
   // This backend cannot express imported calls; exercise its actual invalid
   // scalar source rollback, not a claim of ABI mismatch checking.
   assert!(sessions[0].build("pub fnc main(): int { return missing; }",false).is_err());
   let good=fs::read_to_string(root.join("edit/unit0.atto")).unwrap();assert!(sessions[0].build(&good,false).unwrap().1);
 }
 #[cfg(atto_v3_llvm)]
 if std::env::var_os("ATTO_V3_PARAMETERIZED").is_some(){
  let mut previous=None;
  for (lane,expected) in [("parameterized",42),("parameterized-edit",43)]{
   let source=fs::read_to_string(root.join(lane).join("call0.atto")).unwrap();
   let bad=fs::read_to_string(root.join(lane).join("bad0.atto")).unwrap();
   let (ir,hit)=sessions[0].build(&source,LibraryProfile::FreestandingCore,false).unwrap();assert!(!hit);
   assert_eq!(ir,compile_ir_with_profile(&source,LibraryProfile::FreestandingCore).unwrap());
   if let Some(old)=previous{assert_ne!(old,ir);}
   let path=dir.join("parameterized.ll");let exe=dir.join("parameterized-guest");fs::write(&path,&ir).unwrap();
   assert!(Command::new("clang").arg(&path).arg("-o").arg(&exe).status().unwrap().success());
   assert_eq!(Command::new(&exe).status().unwrap().code(),Some(expected));
   assert!(sessions[0].build(&bad,LibraryProfile::FreestandingCore,false).is_err());
   let (warm,hit)=sessions[0].build(&source,LibraryProfile::FreestandingCore,false).unwrap();assert!(hit);assert_eq!(warm,ir);
   previous=Some(ir);
   println!("V3_LLVM_PARAMETERIZED|{lane}|guest={expected}|warm|arity-denied|rollback");
  }
 }
 println!("V3_NATIVE_UNIT_ROLLBACK|{target}|adapted-not-namespace-linking");
}
