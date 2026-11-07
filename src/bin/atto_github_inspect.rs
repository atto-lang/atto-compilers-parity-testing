//! Read-only GitHub metadata + optional downloaded artifact ZIP checker.
//! Never emits release certification or independent guest execution proof.
use atto_evidence_contract_conformance::artifact::{
    verify_artifact_zip,TrustedReceiptPolicy,MAX_ARTIFACT_ZIP_BYTES
};
use atto_evidence_contract_conformance::github_inspect::{inspect,CurlGitHub,Expected};
use atto_evidence_contract_conformance::decode_strict_json;
use serde_json::{json,Value};
use std::{env,fs,process};

fn policy_file<T: serde::de::DeserializeOwned>(path: &str) -> Result<T,String> {
    let meta=fs::metadata(path).map_err(|_|"cannot inspect trusted policy file")?;
    if meta.len()==0 || meta.len()>1_048_576 {
        return Err("trusted policy exceeds 1 MiB".into());
    }
    let bytes=fs::read(path).map_err(|_|"cannot read trusted policy")?;
    serde_json::from_value(decode_strict_json(&bytes)?)
        .map_err(|_|"invalid trusted verifier policy")
}
fn run() -> Result<Value,String> {
    let args:Vec<_>=env::args().skip(1).collect();
    if args.len()!=1 && !(args.len()==5
        && args[1]=="--artifact-zip" && args[3]=="--receipt-policy") {
        return Err("usage: atto_github_inspect GITHUB_POLICY.json [--artifact-zip ARCHIVE.zip --receipt-policy RECEIPT_POLICY.json]".into());
    }
    let policy:Expected=policy_file(&args[0])?;
    let metadata=inspect(&CurlGitHub::from_environment()?,&policy)?;

    let mut artifact_bytes_verified=false;
    let mut fixture_bytes_verified=false;
    let mut receipt_claim_consistent=false;
    let mut executed_cases_claim=0;
    if args.len()==5 {
        let trusted:TrustedReceiptPolicy=policy_file(&args[4])?;
        if trusted.repository!=metadata.repository ||
            trusted.run_id!=metadata.run_id || trusted.job_id!=metadata.job_id ||
            trusted.source_sha!=metadata.head_sha || trusted.tree_sha!=metadata.tree_sha {
            return Err("artifact policy is inconsistent with independently checked GitHub metadata".into());
        }
        let zip_meta=fs::metadata(&args[2]).map_err(|_|"cannot inspect artifact ZIP")?;
        if zip_meta.len()==0 || zip_meta.len()>MAX_ARTIFACT_ZIP_BYTES as u64 {
            return Err("artifact archive missing or exceeds 8 MiB".into());
        }
        // ZIP bytes must be acquired separately (read-only). Their digest is
        // checked against the *independently fetched* GitHub artifact digest.
        let bytes=fs::read(&args[2]).map_err(|_|"cannot read artifact ZIP")?;
        let verified=verify_artifact_zip(&bytes,&metadata.artifact_digest_claim,&trusted)?;
        artifact_bytes_verified=verified.artifact_bytes_verified;
        fixture_bytes_verified=verified.fixture_bytes_verified;
        receipt_claim_consistent=verified.receipt_claim_consistent;
        executed_cases_claim=verified.executed_cases_claim;
    }
    Ok(json!({"status":"NOT_CERTIFIED","github_metadata":"VERIFIED",
        "repository":metadata.repository,"run_id":metadata.run_id,
        "source_sha":metadata.head_sha,"source_tree_sha":metadata.tree_sha,
        "job_id":metadata.job_id,"runner":metadata.runner_name,
        "artifact_id":metadata.artifact_id,
        "artifact_digest_claim":metadata.artifact_digest_claim,
        "artifact_bytes_verified":artifact_bytes_verified,
        "fixture_bytes_verified":fixture_bytes_verified,
        "receipt_claim_consistent":receipt_claim_consistent,
        "executed_cases_claim":executed_cases_claim,
        "guest_execution_verified":false,
        "reason":"Verified artifact bytes/fixture hashes do not independently prove guest execution"}))
}
fn main() {
    match run() {
        Ok(v)=>println!("{v}"),
        Err(reason)=>{
            println!("{}",json!({"status":"NOT_CERTIFIED","github_metadata":"UNVERIFIED","reason":reason}));
            process::exit(2);
        }
    }
}
