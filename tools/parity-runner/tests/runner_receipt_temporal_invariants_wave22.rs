//! Exact source-owned evidence validator CLI test; no guest compiler execution.
use serde_json::{json, Value};
use std::{fs, process::{Command, Output}, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn evidence() -> Value {
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
            "job_id":31, "job_name":"unit",
            "runner_id":1000002089, "runner_name":"GitHub Actions 1",
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

fn validate(receipt: &Value) -> Output {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "atto-wave22-receipt-{}-{serial}.json", std::process::id()
    ));
    fs::write(&path, serde_json::to_vec(receipt).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_evidence-contract"))
        .arg(&path).output().expect("run evidence-contract CLI");
    fs::remove_file(path).unwrap();
    output
}

#[test]
fn checked_runner_receipt_cannot_be_automatically_promoted_to_guest_tier() {
    let output = validate(&evidence());
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["valid"], true);
    assert_eq!(report["claim_tier"], "RUNNER_EXECUTED");
}

#[test]
fn failed_job_is_not_a_successful_runner_receipt() {
    let mut value = evidence();
    value["jobs"][0]["conclusion"] = json!("failure");
    assert!(!validate(&value).status.success());
}

#[test]
fn failed_meaningful_step_cannot_support_a_successful_run() {
    let mut value = evidence();
    value["jobs"][0]["meaningful_steps"][0]["conclusion"] = json!("failure");
    assert!(!validate(&value).status.success());
}

#[test]
fn declared_verified_job_count_must_match_bound_job_evidence() {
    let mut value = evidence();
    value["verified_jobs"] = json!(2);
    assert!(!validate(&value).status.success());
}

#[test]
fn runnerless_receipt_is_rejected_even_if_job_and_step_claim_success() {
    let mut value = evidence();
    value["jobs"][0]["runner_id"] = Value::Null;
    assert!(!validate(&value).status.success());
}
