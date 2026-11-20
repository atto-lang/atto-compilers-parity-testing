//! Policy gate for proposed hosted std.http/std.fs conformance.
//! This checks source metadata and fails closed on unexecuted evidence.
//! It does NOT run an Atto compiler or create any guest-pass evidence.
use atto_evidence_contract_conformance::release_gate::{
    accept_verified_passing_evidence, ObservedRun, ReleasePolicy,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const MANIFEST: &str = include_str!("../contracts/v1/async-http-fs-parity-v1.json");
const SCHEMA: &str = include_str!("../contracts/v1/conformance-evidence.schema.json");
const SOURCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CASES: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BACKENDS: [&str; 11] = [
    "llvm", "java", "net", "wasm", "rs", "forge", "cranelift",
    "js", "atto", "hdl", "realtime",
];

fn checked_matrix(v: &Value) -> bool {
    if v["schema"] != "atto.feature.parity.std_http_fs.v1"
        || v["feature_id"] != "stdlib.http_fs.roundtrip"
        || v["status"] != "ALL_NOT_EXECUTED"
        || v["guest_required"] != true
        || v["test_endpoint"] != "LOOPBACK_ONLY"
    {
        return false;
    }
    let Some(cases) = v["required_guest_checks"].as_array() else {
        return false;
    };
    let expected = [
        "http_status", "response_text", "file_bytes_equal_body",
        "saved_length", "permission_denial", "redirect_policy",
        "path_escape", "double_consume", "utf8_error",
        "cancellation", "tenant_isolation",
    ];
    let observed: BTreeSet<_> = cases.iter().filter_map(Value::as_str).collect();
    if observed.len() != expected.len() || !expected.iter().all(|x| observed.contains(*x)) {
        return false;
    }
    let Some(backends) = v["backends"].as_array() else {
        return false;
    };
    if backends.len() != BACKENDS.len() {
        return false;
    }
    let mut seen = BTreeSet::new();
    for entry in backends {
        let Some(name) = entry["backend"].as_str() else {
            return false;
        };
        if !BACKENDS.contains(&name)
            || !seen.insert(name)
            || entry["status"] != "NOT_EXECUTED"
            || entry["guest_executed_cases"] != 0
            || entry.as_object().is_none_or(|fields| fields.len() != 3)
        {
            return false;
        }
    }
    seen.len() == BACKENDS.len()
}

#[test]
fn manifest_accounts_for_every_source_backend_without_fake_execution() {
    let value: Value = serde_json::from_str(MANIFEST).unwrap();
    assert!(checked_matrix(&value));
}

#[test]
fn tampering_with_backend_rows_tests_or_status_fails_closed() {
    let original: Value = serde_json::from_str(MANIFEST).unwrap();
    for backend in &BACKENDS {
        let mut changed = original.clone();
        let entry = changed["backends"].as_array_mut().unwrap().iter_mut()
            .find(|e| e["backend"] == *backend).unwrap();
        entry["status"] = json!("PASS");
        assert!(!checked_matrix(&changed), "{backend} forged PASS");
    }
    let mut missing = original.clone();
    missing["backends"].as_array_mut().unwrap().pop();
    assert!(!checked_matrix(&missing));
    let mut duplicated = original.clone();
    duplicated["backends"][10]["backend"] = json!("llvm");
    assert!(!checked_matrix(&duplicated));
    let mut host_only = original.clone();
    host_only["guest_required"] = json!(false);
    assert!(!checked_matrix(&host_only));
    let mut no_file_check = original.clone();
    no_file_check["required_guest_checks"].as_array_mut().unwrap().retain(|x| x != "file_bytes_equal_body");
    assert!(!checked_matrix(&no_file_check));
}

#[test]
fn unexecuted_or_host_only_claims_cannot_unlock_release_for_any_backend() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    for backend in BACKENDS {
        let policy = ReleasePolicy {
            trusted_repository: "atto-lang/atto-compilers-parity-testing",
            source_sha: SOURCE,
            case_set_sha256: CASES,
            backend,
            feature_id: "stdlib.http_fs.roundtrip",
            require_guest: true,
        };
        let observed = ObservedRun {
            repository: "atto-lang/atto-compilers-parity-testing",
            run_id: 42,
            head_sha: SOURCE,
            complete: false,
            workflow_conclusion: "skipped",
            test_job_conclusion: "skipped",
            test_steps_executed: false,
            execution_target: "host",
            executed_cases: 0,
            failed_cases: 0,
            case_set_sha256: CASES,
            artifact_digests_verified: false,
            independent_reviews_verified: false,
            protected_main_verified: false,
        };
        let report = json!({
            "version": 1,
            "backend": backend,
            "feature_id": "stdlib.http_fs.roundtrip",
            "source_sha": SOURCE,
            "case_set_sha256": CASES,
            "status": "not_executed",
            "execution_target": "host",
            "run_url": "https://github.com/atto-lang/atto-compilers-parity-testing/actions/runs/42",
            "executed_cases": 0,
            "failed_cases": 0
        });
        assert!(accept_verified_passing_evidence(
            report.to_string().as_bytes(), &schema, &policy, &observed
        ).is_err(), "{backend}: unexecuted report");
        let mut forged = report.clone();
        forged["status"] = json!("passed");
        forged["execution_target"] = json!("guest");
        assert!(accept_verified_passing_evidence(
            forged.to_string().as_bytes(), &schema, &policy, &observed
        ).is_err(), "{backend}: forged guest report");
    }
}
