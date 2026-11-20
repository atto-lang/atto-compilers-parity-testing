//! Source-owned evidence-contract CLI tests, NOT compiled guest parity.
use serde_json::{json,Value};
use std::{fs,process::{Command,Output},sync::atomic::{AtomicU64,Ordering}};
static NEXT:AtomicU64=AtomicU64::new(1);

fn base() -> Value {
    json!({
        "schema":"oreslang-parity-execution-evidence-v1",
        "claim_tier":"RUNNER_EXECUTED",
        "backend":"java",
        "repository":"ores-truffle-oreslang/oreslang-source.java",
        "repository_id":1400792274u64,
        "run_id":17,
        "run_attempt":2,
        "workflow_id":991,
        "workflow_path":".github/workflows/ci.yml",
        "workflow_run_path":".github/workflows/ci.yml",
        "event":"push",
        "head_commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "head_tree":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "workflow_blob_sha":"cccccccccccccccccccccccccccccccccccccccc",
        "verified_jobs":1,
        "successful_non_housekeeping_steps":1,
        "jobs":[{
            "job_id":31,
            "job_name":"unit",
            "runner_id":1000002089,
            "runner_name":"GitHub Actions 1",
            "conclusion":"success",
            "started_at":"2026-10-16T10:00:00Z",
            "completed_at":"2026-10-16T10:03:00Z",
            "meaningful_steps":[{
                "name":"Execute test suite",
                "category":"indeterminate",
                "conclusion":"success",
                "started_at":"2026-10-16T10:01:00Z",
                "completed_at":"2026-10-16T10:02:00Z"
            }]
        }],
        "artifacts_verified":false,
        "artifacts":[],
        "statement":"GITHUB_RUN_EXECUTION_VERIFIED_NOT_GUEST_PARITY_CERTIFIED"
    })
}

fn run(value:&Value)->Output {
    let n=NEXT.fetch_add(1,Ordering::Relaxed);
    let file=std::env::temp_dir().join(format!(
        "atto-wave19-runner-evidence-{}-{n}.json",std::process::id()));
    fs::write(&file,serde_json::to_vec(value).unwrap()).unwrap();
    let output=Command::new(env!("CARGO_BIN_EXE_evidence-contract"))
        .arg(&file).output().unwrap();
    fs::remove_file(file).unwrap();
    output
}

#[test]
fn valid_hosted_runner_receipt_remains_runner_only() {
    let output=run(&base());
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    let report:Value=serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["valid"],true);
    assert_eq!(report["claim_tier"],"RUNNER_EXECUTED");
}

#[test]
fn runner_admission_failure_cannot_be_rebranded_as_execution() {
    for (name,mut value) in [
        ("zero",base()),("missing",base()),("missing-steps",base())
    ] {
        match name {
            "zero"=>value["jobs"][0]["runner_id"]=json!(0),
            "missing"=>value["jobs"][0]["runner_id"]=Value::Null,
            "missing-steps"=>value["jobs"][0]["meaningful_steps"]=json!([]),
            _=>unreachable!()
        }
        assert!(!run(&value).status.success(),"false runner admission: {name}");
    }
}

#[test]
fn housekeeping_only_and_zero_step_count_never_prove_execution() {
    let mut housekeeping=base();
    housekeeping["jobs"][0]["meaningful_steps"][0]["name"]=json!("Post checkout");
    assert!(!run(&housekeeping).status.success());
    let mut absent=base();
    absent["successful_non_housekeeping_steps"]=json!(0);
    assert!(!run(&absent).status.success());
}

#[test]
fn claim_tiers_cannot_be_self_promoted_without_bound_guest_report() {
    let mut host=base();
    host["claim_tier"]=json!("HOST_TESTS_EXECUTED");
    assert!(!run(&host).status.success());
    let mut guest=base();
    guest["claim_tier"]=json!("ORES_GUEST_EXECUTED");
    assert!(!run(&guest).status.success());
}

#[test]
fn source_backend_repository_identity_is_nonfungible() {
    let mut forged=base();
    forged["backend"]=json!("llvm");
    assert!(!run(&forged).status.success());
    let mut wrong_repo=base();
    wrong_repo["repository_id"]=json!(1400792275u64);
    assert!(!run(&wrong_repo).status.success());
}
