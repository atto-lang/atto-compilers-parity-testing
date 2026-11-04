use serde_json::{json, Value};
use std::process::Command;

fn base(backend: &str) -> Value {
    let (repository, repository_id) = match backend {
        "java" => ("ores-truffle-oreslang/oreslang-source.java", 1400792274u64),
        "llvm" => ("ores-truffle-oreslang/oreslang-source.llvm", 1410897874u64),
        "net" => ("ores-truffle-oreslang/oreslang-source.net", 1419704375u64),
        "wasm" => ("ores-truffle-oreslang/oreslang-source.wasm", 1419858954u64),
        "forge" => ("ores-truffle-oreslang/oreslang-source.forge", 1419703674u64),
        "cranelift" => ("ores-truffle-oreslang/oreslang-source.cranelift", 1437273004u64),
        _ => panic!("bad test backend"),
    };
    json!({
        "schema": "oreslang-parity-execution-evidence-v1",
        "claim_tier": "RUNNER_EXECUTED",
        "backend": backend,
        "repository": repository,
        "repository_id": repository_id,
        "run_id": 17,
        "run_attempt": 2,
        "workflow_id": 991,
        "workflow_path": ".github/workflows/ci.yml",
        "workflow_run_path": ".github/workflows/ci.yml",
        "event": "push",
        "head_commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "head_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "workflow_blob_sha": "cccccccccccccccccccccccccccccccccccccccc",
        "verified_jobs": 1,
        "successful_non_housekeeping_steps": 1,
        "jobs": [{
            "job_id": 31,
            "job_name": "unit",
            "runner_id": 1000002089,
            "runner_name": "GitHub Actions 1",
            "conclusion": "success",
            "started_at": "2026-10-16T10:00:00Z",
            "completed_at": "2026-10-16T10:03:00Z",
            "meaningful_steps": [{
                "name": "Execute test suite",
                "category": "indeterminate",
                "conclusion": "success",
                "started_at": "2026-10-16T10:01:00Z",
                "completed_at": "2026-10-16T10:02:00Z"
            }]
        }],
        "artifacts_verified": false,
        "artifacts": [],
        "statement": "GITHUB_RUN_EXECUTION_VERIFIED_NOT_GUEST_PARITY_CERTIFIED"
    })
}

fn run_with_report(name: &str, value: &Value, report: Option<&Value>) -> std::process::Output {
    let path = std::env::temp_dir().join(format!(
        "ores-evidence-contract-{}-{name}.json",
        std::process::id()
    ));
    std::fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_evidence-contract"));
    command.arg(&path);
    let report_path = report.map(|report| {
        let path = std::env::temp_dir().join(format!(
            "ores-evidence-report-{}-{name}.json",
            std::process::id()
        ));
        std::fs::write(&path, serde_json::to_vec_pretty(report).unwrap()).unwrap();
        command.arg("--report").arg(&path);
        path
    });
    let output = command.output().unwrap();
    std::fs::remove_file(path).unwrap();
    if let Some(path) = report_path {
        std::fs::remove_file(path).unwrap();
    }
    output
}

fn run(name: &str, value: &Value) -> std::process::Output {
    run_with_report(name, value, None)
}

#[test]
fn valid_runner_execution_is_accepted_and_net_normalizes_to_dotnet() {
    let output = run("net-valid", &base("net"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["valid"], true);
    assert_eq!(result["source_backend"], "net");
    assert_eq!(result["parity_backend"], "dotnet");
    assert_eq!(result["claim_tier"], "RUNNER_EXECUTED");
    assert_eq!(result["run_attempt"], 2);
}

#[test]
fn zero_runner_cannot_be_execution_evidence() {
    let mut value = base("java");
    value["jobs"][0]["runner_id"] = json!(0);
    assert!(!run("zero-runner", &value).status.success());
}

#[test]
fn housekeeping_only_step_cannot_be_execution_evidence() {
    let mut value = base("llvm");
    value["jobs"][0]["meaningful_steps"][0]["name"] = json!("Post checkout");
    assert!(!run("housekeeping", &value).status.success());
}

#[test]
fn repository_id_must_match_backend_identity() {
    let mut value = base("wasm");
    value["repository_id"] = json!(1419858955u64);
    assert!(!run("wrong-repo", &value).status.success());
}

#[test]
fn host_tier_cannot_be_self_promoted_without_host_test_step() {
    let mut value = base("forge");
    value["claim_tier"] = json!("HOST_TESTS_EXECUTED");
    assert!(!run("host-tier", &value).status.success());
}

fn guest_bundle() -> Value {
    let mut value = base("llvm");
    value["claim_tier"] = json!("ORES_GUEST_EXECUTED");
    value["successful_non_housekeeping_steps"] = json!(3);
    value["jobs"][0]["meaningful_steps"] = json!([
        {
            "name": "Run host compiler tests",
            "category": "host_compiler_tests",
            "conclusion": "success",
            "started_at": "2026-10-16T10:00:20Z",
            "completed_at": "2026-10-16T10:00:50Z"
        },
        {
            "name": "Build native compiler",
            "category": "native_compiler_build",
            "conclusion": "success",
            "started_at": "2026-10-16T10:01:00Z",
            "completed_at": "2026-10-16T10:01:40Z"
        },
        {
            "name": "Execute compiled Oreslang guest",
            "category": "compiled_ores_guest_runtime_execution",
            "conclusion": "success",
            "started_at": "2026-10-16T10:02:00Z",
            "completed_at": "2026-10-16T10:02:40Z"
        }
    ]);
    value["artifacts_verified"] = json!(true);
    value["artifacts"] = json!([
        {
            "id": 501,
            "name": "compiler",
            "expired": false,
            "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        },
        {
            "id": 502,
            "name": "guest",
            "expired": false,
            "sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
        }
    ]);
    value["compiler"] = json!({
        "artifact_id": 501,
        "identity": "oresc llvm exact-head",
        "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "source_head_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    value["guest"] = json!({
        "case_id": "checked-i64-smoke",
        "source_sha256": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "artifact_id": 502,
        "artifact_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        "exit_code": 42,
        "expected_exit_code": 42,
        "stdout_hex": "34320a",
        "expected_stdout_hex": "34320a"
    });
    value
}

fn guest_report() -> Value {
    json!({
        "schema_version": 2,
        "executed": true,
        "backends": ["llvm"],
        "records": [{
            "suite": "legacy",
            "case_id": "checked-i64-smoke",
            "backend": "llvm",
            "source_sha256": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            "status": "pass",
            "validation_status": "validated",
            "compile": {
                "status": "completed",
                "exit_code": 0
            },
            "run": {
                "status": "completed",
                "exit_code": 42,
                "stdout_hex": "34320a"
            }
        }]
    })
}

#[test]
fn full_compiled_guest_bundle_with_exact_oracle_is_accepted() {
    let evidence = guest_bundle();
    let report = guest_report();
    let output = run_with_report("guest-valid", &evidence, Some(&report));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["claim_tier"], "ORES_GUEST_EXECUTED");
    assert_eq!(result["parity_backend"], "llvm");
}

#[test]
fn compiled_guest_tier_requires_parity_report_binding() {
    assert!(!run("guest-no-report", &guest_bundle()).status.success());
}

#[test]
fn guest_oracle_that_disagrees_with_parity_report_is_rejected() {
    let mut evidence = guest_bundle();
    evidence["guest"]["exit_code"] = json!(41);
    evidence["guest"]["expected_exit_code"] = json!(41);
    let report = guest_report();
    assert!(
        !run_with_report("guest-report-mismatch", &evidence, Some(&report))
            .status
            .success()
    );
}

#[test]
fn compiler_or_guest_tier_requires_verified_artifact_enumeration() {
    let mut value = guest_bundle();
    value["artifacts_verified"] = json!(false);
    let report = guest_report();
    assert!(
        !run_with_report("artifacts-unverified", &value, Some(&report))
            .status
            .success()
    );
}

#[test]
fn expired_compiler_artifact_cannot_support_native_or_guest_claim() {
    let mut value = guest_bundle();
    value["artifacts"][0]["expired"] = json!(true);
    assert!(!run("expired-compiler", &value).status.success());
}

#[test]
fn duplicate_job_identity_is_rejected() {
    let mut value = base("java");
    let duplicate = value["jobs"][0].clone();
    value["jobs"].as_array_mut().unwrap().push(duplicate);
    value["verified_jobs"] = json!(2);
    value["successful_non_housekeeping_steps"] = json!(2);
    assert!(!run("duplicate-job", &value).status.success());
}

#[test]
fn impossible_utc_calendar_instants_cannot_prove_execution() {
    // Equal endpoints isolate date validity from chronological checks.
    for (index, stamp) in [
        "0000-01-01T12:00:00Z",
        "2026-00-01T12:00:00Z",
        "2026-13-01T12:00:00Z",
        "2026-01-00T12:00:00Z",
        "2026-02-29T12:00:00Z",
        "2026-04-31T12:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T12:60:00Z",
        "2026-01-01T12:00:60Z",
        "1900-02-29T12:00:00Z",
    ]
    .iter()
    .enumerate()
    {
        let mut evidence = base("java");
        evidence["jobs"][0]["started_at"] = json!(stamp);
        evidence["jobs"][0]["completed_at"] = json!(stamp);
        evidence["jobs"][0]["meaningful_steps"][0]["started_at"] = json!(stamp);
        evidence["jobs"][0]["meaningful_steps"][0]["completed_at"] = json!(stamp);
        let output = run(&format!("invalid-utc-{index}"), &evidence);
        assert!(!output.status.success(), "accepted invalid date: {stamp}");
    }
    let mut leap = base("java");
    for field in ["started_at", "completed_at"] {
        leap["jobs"][0][field] = json!("2024-02-29T12:00:00Z");
        leap["jobs"][0]["meaningful_steps"][0][field] = json!("2024-02-29T12:00:00Z");
    }
    let output = run("valid-leap-day", &leap);
    assert!(
        output.status.success(),
        "rejected valid leap day: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn guest_report_cannot_self_promote_unvalidated_or_uncompiled_cases() {
    let evidence = guest_bundle();
    let mut report = guest_report();
    report["records"][0]["validation_status"] = json!("candidate");
    assert!(
        !run_with_report("guest-candidate-fake-pass", &evidence, Some(&report))
            .status
            .success()
    );

    let mut report = guest_report();
    report["records"][0]["compile"]["status"] = json!("timeout");
    assert!(
        !run_with_report("guest-compile-timeout", &evidence, Some(&report))
            .status
            .success()
    );

    let mut report = guest_report();
    report["records"][0]["compile"]["exit_code"] = json!(1);
    assert!(
        !run_with_report("guest-compile-error", &evidence, Some(&report))
            .status
            .success()
    );

    let mut report = guest_report();
    report["schema_version"] = json!(3);
    assert!(
        !run_with_report("guest-wrong-schema", &evidence, Some(&report))
            .status
            .success()
    );
}

#[test]
fn valid_cranelift_runner_execution_is_distinct_from_forge_and_guest_proof() {
    let output = run("cranelift-valid", &base("cranelift"));
    assert!(output.status.success(), "{:?}", String::from_utf8_lossy(&output.stderr));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["source_backend"], "cranelift");
    assert_eq!(result["parity_backend"], "cranelift");
    assert_eq!(result["claim_tier"], "RUNNER_EXECUTED");
}
#[test]
fn cranelift_identity_cannot_be_spoofed_with_forge_repository_id() {
    let mut value = base("cranelift");
    value["repository_id"] = json!(1419703674);
    assert!(!run("cranelift-forge-id", &value).status.success());
    value["repository_id"] = json!(1437273004);
    value["repository"] = json!("ores-truffle-oreslang/oreslang-source.forge");
    assert!(!run("cranelift-forge-repo", &value).status.success());
}
