use atto_evidence_contract_conformance::artifact_receipt::{
    ArtifactPolicy,ProvenanceVerifier,ReplayFence,validate_archive,validate_signed,sha256_hex,MAX_ARCHIVE_BYTES
};
use serde_json::{json,Value};
use std::io::{Cursor,Write};
use zip::{ZipWriter,write::SimpleFileOptions,CompressionMethod};
const SOURCE:&str="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE:&str="cccccccccccccccccccccccccccccccccccccccc";
const CASES:&str="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const NONCE:&str="0123456789abcdef0123456789abcdef";
const COMMAND:&str="atto guest --conformance actor.ownership";
struct FakeSigner(bool);
impl ProvenanceVerifier for FakeSigner{
    fn verify_signed(&self,_:&[u8],_:&ArtifactPolicy<'_>)->Result<(),String>{
        if self.0{Ok(())}else{Err("signature missing".into())}
    }
}
fn policy<'a>(digest:&'a str)->ArtifactPolicy<'a>{
    ArtifactPolicy{repository:"benefactor-cc/ci",source_sha:SOURCE,tree_sha:TREE,
        case_sha256:CASES,backend:"llvm",feature:"actor.ownership",run_id:42,
        run_attempt:2,job_id:123,artifact_id:400,artifact_digest:digest,
        command:COMMAND,nonce:NONCE,signer_workflow:".github/workflows/guest.yml"}
}
fn receipt()->Value{
    json!({"version":1,"repository":"benefactor-cc/ci","source_sha":SOURCE,
        "tree_sha":TREE,"case_sha256":CASES,"backend":"llvm","feature":"actor.ownership",
        "run_id":42,"run_attempt":2,"job_id":123,"target":"guest",
        "command":COMMAND,"nonce":NONCE,"executed":7,"failed":0,
        "exit_code":0,"stdout_sha256":sha256_hex(b"7 cases executed\n")})
}
fn archive(r:&Value,stdout:&[u8])->Vec<u8>{
    let mut w=ZipWriter::new(Cursor::new(Vec::new()));
    let opts=SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    w.start_file("guest-receipt.json",opts).unwrap();
    w.write_all(r.to_string().as_bytes()).unwrap();
    w.start_file("guest-stdout.bin",opts).unwrap();
    w.write_all(stdout).unwrap();
    w.finish().unwrap().into_inner()
}
fn acceptable(r:&Value,stdout:&[u8])->bool{
    let bytes=archive(r,stdout);
    let d=format!("sha256:{}",sha256_hex(&bytes));
    validate_archive(&bytes,&policy(&d)).is_ok()
}
#[test]fn valid_zip_bytes_and_captured_output_are_verified(){
    let bytes=archive(&receipt(),b"7 cases executed\n");
    let d=format!("sha256:{}",sha256_hex(&bytes));
    assert_eq!(validate_archive(&bytes,&policy(&d)).unwrap().receipt.executed,7);
}
#[test]fn swapped_zip_bytes_and_fake_digests_rejected(){
    let bytes=archive(&receipt(),b"7 cases executed\n");
    assert!(validate_archive(&bytes,&policy(&format!("sha256:{}", "f".repeat(64)))).is_err());
    let mut fake=bytes.clone();fake[0]^=1;
    let d=format!("sha256:{}",sha256_hex(&bytes));
    assert!(validate_archive(&fake,&policy(&d)).is_err());
    assert!(validate_archive(&vec![0;MAX_ARCHIVE_BYTES+1],&policy(&d)).is_err());
}
#[test]fn stale_host_only_failed_and_replayed_guest_claims_rejected(){
    for (key,bad) in [
        ("version",json!(2)),("repository",json!("evil/ci")),
        ("source_sha",json!("d".repeat(40))),("tree_sha",json!("d".repeat(40))),
        ("case_sha256",json!("e".repeat(64))),("backend",json!("java")),
        ("feature",json!("actor.other")),("run_id",json!(43)),
        ("run_attempt",json!(1)),("job_id",json!(456)),("target",json!("host")),
        ("command",json!("cargo test")),("nonce",json!("wrong_nonce")),
        ("executed",json!(0)),("failed",json!(1)),("exit_code",json!(1)),
        ("stdout_sha256",json!("f".repeat(64)))
    ]{
        let mut r=receipt();r[key]=bad;
        assert!(!acceptable(&r,b"7 cases executed\n"),"accepted {key}");
    }
    let mut r=receipt();r["extra"]=json!(true);
    assert!(!acceptable(&r,b"7 cases executed\n"));
    assert!(!acceptable(&receipt(),b"spoofed output"));
}
#[test]fn unsigned_archive_rejected_and_batch_replay_blocked(){
    let bytes=archive(&receipt(),b"7 cases executed\n");
    let d=format!("sha256:{}",sha256_hex(&bytes));
    assert!(validate_signed(&bytes,&policy(&d),&FakeSigner(false)).is_err());
    let checked=validate_signed(&bytes,&policy(&d),&FakeSigner(true)).unwrap();
    let mut fence=ReplayFence::default();
    assert!(fence.admit(&checked).is_ok());
    assert!(fence.admit(&checked).is_err());
}
#[test]fn rejects_unknown_paths_zip_members_and_zip_bombs(){
    let mut w=ZipWriter::new(Cursor::new(Vec::new()));
    for path in ["../guest-receipt.json","guest-stdout.bin"]{
        w.start_file(path,SimpleFileOptions::default()).unwrap();
        w.write_all(b"{}").unwrap();
    }
    let bytes=w.finish().unwrap().into_inner();
    let d=format!("sha256:{}",sha256_hex(&bytes));
    assert!(validate_archive(&bytes,&policy(&d)).is_err());
    let bytes=archive(&receipt(),&vec![0u8;1024*1024+1]);
    let d=format!("sha256:{}",sha256_hex(&bytes));
    assert!(validate_archive(&bytes,&policy(&d)).is_err());
}
