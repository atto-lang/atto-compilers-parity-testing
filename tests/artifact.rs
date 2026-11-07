use atto_evidence_contract_conformance::artifact::{
    verify_artifact_zip,TrustedReceiptPolicy,MAX_ARTIFACT_ZIP_BYTES
};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::io::{Cursor,Write};
use zip::{write::SimpleFileOptions,CompressionMethod,ZipWriter};

fn digest(x:&[u8])->String {format!("sha256:{:x}",Sha256::digest(x))}
fn policy()->TrustedReceiptPolicy {
    TrustedReceiptPolicy {
        repository:"benefactor-cc/atto-guest-ci".into(),
        run_id:42,job_id:20,
        source_sha:"a".repeat(40),tree_sha:"b".repeat(40),
        backend:"llvm".into(),feature_id:"actor.ownership".into(),
        case_set_sha256:format!("{:x}",Sha256::digest(br#"[{"case":"ownership"}]"#)),
    }
}
fn receipt(policy:&TrustedReceiptPolicy)->Value {
    json!({"version":1,"repository":policy.repository,
        "run_id":policy.run_id,"job_id":policy.job_id,
        "source_sha":policy.source_sha,"tree_sha":policy.tree_sha,
        "backend":policy.backend,"feature_id":policy.feature_id,
        "case_set_sha256":policy.case_set_sha256,
        "status":"passed","execution_target":"guest",
        "executed_cases":1,"failed_cases":0})
}
fn archive(entries:&[(&str,&[u8])])->Vec<u8> {
    let mut zip=ZipWriter::new(Cursor::new(Vec::<u8>::new()));
    for (name,bytes) in entries {
        zip.start_file(*name,SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
fn build(r:&[u8],fixture:&[u8])->Vec<u8> {
    archive(&[("guest-receipt.json",r),("fixture-set.json",fixture)])
}
fn valid()->(TrustedReceiptPolicy,Vec<u8>) {
    let p=policy();
    let bytes=build(receipt(&p).to_string().as_bytes(),br#"[{"case":"ownership"}]"#);
    (p,bytes)
}
fn deny(bytes:&[u8],p:&TrustedReceiptPolicy) {
    assert!(verify_artifact_zip(bytes,&digest(bytes),p).is_err());
}
#[test]
fn validates_real_zip_sha256_and_fixture_bytes_but_never_claims_guest_proof(){
    let (p,data)=valid();
    let r=verify_artifact_zip(&data,&digest(&data),&p).unwrap();
    assert!(r.artifact_bytes_verified && r.fixture_bytes_verified && r.receipt_claim_consistent);
    assert_eq!(r.executed_cases_claim,1);
    assert!(!r.guest_execution_verified);
}
#[test]
fn rejects_forged_or_malformed_github_artifact_digest(){
    let(p,data)=valid();
    assert!(verify_artifact_zip(&data,"sha256:abcdef",&p).is_err());
    assert!(verify_artifact_zip(&data,&format!("sha256:{}", "0".repeat(64)),&p).is_err());
    assert!(verify_artifact_zip(&data,"md5:0123456789",&p).is_err());
    assert!(verify_artifact_zip(&data,"",&p).is_err());
}
#[test]
fn rejects_untrusted_archive_member_paths_duplicates_or_missing_files(){
    let p=policy();let r=receipt(&p).to_string();
    for name in ["../guest-receipt.json","/guest-receipt.json",
        "subdir/guest-receipt.json","guest-receipt.json/","guest-receipt.JSON"] {
        let zip=archive(&[(name,r.as_bytes()),("fixture-set.json",br#"[{"case":"ownership"}]"#)]);
        deny(&zip,&p);
    }
    let zip=archive(&[("guest-receipt.json",r.as_bytes())]);deny(&zip,&p);
    let zip=archive(&[("guest-receipt.json",r.as_bytes()),
        ("fixture-set.json",br#"[{"case":"ownership"}]"#),
        ("extra.txt",b"extra")]);deny(&zip,&p);
    // ZipWriter rejects duplicate names at construction. Build a valid ZIP
    // with same-length distinct names, then patch both local + central names
    // to create the malformed duplicate-entry archive intentionally.
    let mut zip=archive(&[("guest-receipt.json",r.as_bytes()),
        ("guest-receip2.json",r.as_bytes())]);
    let from=b"guest-receip2.json";let to=b"guest-receipt.json";
    assert_eq!(from.len(),to.len());
    let mut patched=0;
    for offset in 0..zip.len()-from.len() {
        if &zip[offset..offset+from.len()]==from {
            zip[offset..offset+from.len()].copy_from_slice(to);
            patched+=1;
        }
    }
    assert_eq!(patched,2); // one local header and one central-directory header
    deny(&zip,&p);
}
#[test]
fn rejects_receipt_mutations_while_github_digest_is_recalculated(){
    let p=policy();
    for (field,bad) in [
        ("version",json!(2)),("repository",json!("attacker/ci")),
        ("run_id",json!(41)),("job_id",json!(21)),
        ("source_sha",json!("e".repeat(40))),("tree_sha",json!("e".repeat(40))),
        ("backend",json!("wasm")),("feature_id",json!("different")),
        ("case_set_sha256",json!("0".repeat(64))),("status",json!("failed")),
        ("status",json!("not_executed")),("status",json!("unsupported")),
        ("execution_target",json!("host")),("executed_cases",json!(0)),
        ("failed_cases",json!(1)),("executed_cases",json!(2)),
    ] {
        let mut r=receipt(&p);r[field]=bad;
        let zip=build(r.to_string().as_bytes(),br#"[{"case":"ownership"}]"#);
        deny(&zip,&p);
    }
}
#[test]
fn rejects_fixture_bytes_hash_mismatch_and_non_json_fixtures(){
    let p=policy();
    let r=receipt(&p).to_string();
    for cases in [br#"[{"case":"different"}]"#.as_slice(),b"{}".as_slice(),
        b"\xff".as_slice(),br#"[{"x":1,"x":2}]"#] {
        deny(&build(r.as_bytes(),cases),&p);
    }
    let mut alt=policy();
    alt.case_set_sha256=format!("{:x}",Sha256::digest(b"{}"));
    let zip=build(receipt(&alt).to_string().as_bytes(),b"{}");
    deny(&zip,&alt); // actual fixture bytes hash correctly but type must be JSON array
}
#[test]
fn rejects_duplicate_keys_unknown_fields_and_float_syntax(){
    let p=policy();
    let r=receipt(&p).to_string();
    let r=r.trim_end_matches('}');
    for suffix in [",\"status\":\"passed\"}",",\"extra\":true}",",\"executed_cases\":1.0}"] {
        deny(&build(format!("{r}{suffix}").as_bytes(),br#"[{"case":"ownership"}]"#),&p);
    }
}
#[test]
fn enforces_compressed_and_decompressed_resource_budgets(){
    let p=policy();let r=receipt(&p).to_string();
    let bad=build(r.as_bytes(),&vec![b'x';4*1024*1024+1]);
    deny(&bad,&p);
    let huge=build(&vec![b' ';1024*1024+1],br#"[{"case":"ownership"}]"#);
    deny(&huge,&p);
    assert!(verify_artifact_zip(&vec![0u8;MAX_ARTIFACT_ZIP_BYTES+1],
        &format!("sha256:{}", "0".repeat(64)),&p).is_err());
}
#[test]
fn rejects_invalid_trees_run_ids_and_corrupted_zip_bytes(){
    let (mut p,bytes)=valid();p.tree_sha="INVALID".into();deny(&bytes,&p);
    let (mut p,bytes)=valid();p.job_id=0;deny(&bytes,&p);
    let (p,mut bytes)=valid();
    let mid=bytes.len()/2; bytes[mid]^=1;
    deny(&bytes,&p);
    deny(b"not a ZIP",&p);
}
