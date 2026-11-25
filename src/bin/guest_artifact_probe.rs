//! Read-only, fail-closed combined metadata/artifact-byte/attestation probe.
//! Even successful signed provenance is NOT_CERTIFIED without independently
//! established runner execution and protected-source governance.
use atto_evidence_contract_conformance::{
    github_probe::{probe_github_metadata,GhCli,ProbeSpec},
    artifact_receipt::{ArtifactPolicy,fetch_and_verify,DurableReplayFence}
};
use serde_json::json;
fn main(){
    let args:Vec<String>=std::env::args().collect();
    if args.len()!=17{
        eprintln!("usage: guest_artifact_probe REPO PR RUN HEAD_SHA TREE_SHA WORKFLOW JOB REVIEWER ARTIFACT STEP1 STEP2 CASE_SHA256 BACKEND FEATURE NONCE GUEST_COMMAND");
        std::process::exit(2);
    }
    let pr=args[2].parse().unwrap_or(0);
    let run=args[3].parse().unwrap_or(0);
    let steps=[args[10].as_str(),args[11].as_str()];
    let s=ProbeSpec{
        repository:&args[1],pr_number:pr,run_id:run,source_sha:&args[4],
        source_tree_sha:&args[5],workflow_path:&args[6],job_name:&args[7],
        approved_reviewer:&args[8],artifact_name:&args[9],required_steps:&steps,
    };
    let attempt=||->Result<serde_json::Value,String>{
        let observed=probe_github_metadata(&GhCli,&s)?;
        let policy=ArtifactPolicy{
            repository:&args[1],source_sha:&args[4],tree_sha:&args[5],
            case_sha256:&args[12],backend:&args[13],feature:&args[14],
            run_id:observed.run_id,run_attempt:observed.run_attempt,
            job_id:observed.job_id,artifact_id:observed.artifact_id,
            artifact_digest:&observed.artifact_digest_metadata,
            command:&args[16],nonce:&args[15],signer_workflow:&args[6],
        };
        let signed=fetch_and_verify(&policy)?;
        // Explicit private local journal is mandatory: accepting the same
        // receipt in a second process must fail even if its signature is valid.
        let journal=std::env::var("ATTO_REPLAY_JOURNAL_DIR")
            .map_err(|_|"ATTO_REPLAY_JOURNAL_DIR must name a private local replay directory")?;
        DurableReplayFence::open(journal)?.admit(&signed)?;
        Ok(json!({
            "status":"NOT_CERTIFIED",
            "metadata":"CHECKED","actual_archive_bytes":"SHA256_VERIFIED",
            "signed_provenance":"VERIFIED",
            "guest_receipt":"STRUCTURALLY_VERIFIED_CLAIM",
            "trusted_guest_execution":"NOT_ESTABLISHED",
            "repository":observed.repository,
            "source_sha":observed.source_sha,
            "tree_sha":observed.tree_sha,
            "run_id":observed.run_id,
            "run_attempt":observed.run_attempt,
            "job_id":observed.job_id,
            "artifact_id":observed.artifact_id,
            "archive_sha256":signed.archive_sha256,
            "executed_cases_claimed":signed.receipt.executed,
        }))
    };
    match attempt(){
        Ok(value)=>{
            println!("{}",serde_json::to_string_pretty(&value).expect("serialize"));
            std::process::exit(3);
        }
        Err(error)=>{
            eprintln!("NOT_CERTIFIED: {error}");
            std::process::exit(2);
        }
    }
}
