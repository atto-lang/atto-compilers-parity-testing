//! Read-only GitHub metadata inspector. It never emits release certification.
use atto_evidence_contract_conformance::github_inspect::{inspect,CurlGitHub,Expected};
use serde_json::{json, Value};
use std::{env,fs,process};
fn run() -> Result<Value,String> {
    let mut args=env::args(); let _program=args.next();
    let input=args.next().ok_or("expected exactly one trusted policy JSON path")?;
    if args.next().is_some(){return Err("expected one trusted policy JSON path".into());}
    let raw=fs::read(input).map_err(|_|"cannot read trusted policy JSON")?;
    let parsed=atto_evidence_contract_conformance::decode_strict_json(&raw)?;
    let policy:Expected=serde_json::from_value(parsed).map_err(|_|"invalid policy schema")?;
    let receipt=inspect(&CurlGitHub::from_environment()?,&policy)?;
    Ok(json!({"status":"NOT_CERTIFIED","github_metadata":"VERIFIED",
        "repo":receipt.repository,"run_id":receipt.run_id,"source_sha":receipt.head_sha,
        "source_tree_sha":receipt.tree_sha,"job_id":receipt.job_id,
        "runner":receipt.runner_name,"artifact_id":receipt.artifact_id,
        "artifact_digest_claim":receipt.artifact_digest_claim,
        "artifact_bytes_verified":receipt.artifact_bytes_verified,
        "guest_execution_verified":receipt.guest_execution_verified,
        "reason":"GitHub API metadata is not guest execution nor artifact-byte proof"}))
}
fn main(){
    match run(){
        Ok(v)=>println!("{v}"),
        Err(reason)=>{
            println!("{}",json!({"status":"NOT_CERTIFIED","github_metadata":"UNVERIFIED","reason":reason}));
            process::exit(2);
        }
    }
}
