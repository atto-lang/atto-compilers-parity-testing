//! Full-process adapter fault injection using shell stand-ins, not real
//! compiler execution or evidence of guest parity.
#![cfg(unix)]

use serde_json::Value;
use std::{
    fs,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run_with_adapters(scenario: &str, compiler: &str, guest: &str) -> (Output, Value) {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "ores-parity-adapter-e2e-{}-{scenario}-{serial}.json",
        std::process::id()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args([
            "--backend",
            "java",
            "--execute",
            "--allow-unvalidated",
            "--timeout",
            "2",
            "--output",
        ])
        .arg(&path)
        .env("ORES_PARITY_JAVA_COMPILE", compiler)
        .env("ORES_PARITY_JAVA_RUN", guest)
        .output()
        .expect("execute parity runner with synthetic adapters");
    let report: Value = serde_json::from_slice(&fs::read(&path).expect("written report"))
        .expect("valid report JSON");
    fs::remove_file(path).expect("remove report");
    assert_eq!(report["cases"], 54);
    assert_eq!(report["records"].as_array().unwrap().len(), 54);
    assert_eq!(report["backends"], serde_json::json!(["java"]));
    (output, report)
}

fn case<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["case_id"] == id)
        .expect("case must exist in inventory")
}

#[test]
fn generic_compiler_errors_cannot_pass_negative_probes() {
    let (output, report) = run_with_adapters(
        "generic-reject",
        r#"sh -c 'echo compiler-failure >&2; exit 1'"#,
        r#"sh -c 'exit 0'"#,
    );
    assert!(!output.status.success());
    let negative = case(&report, "nb-match/negative-1");
    assert_eq!(negative["status"], "rejection_unclassified");
    assert!(negative["run"].is_null());
    assert_eq!(case(&report, "actor-system")["status"], "compile_failure");
    assert!(report["summary"].get("pass").is_none());
    assert!(report["summary"].get("expected_reject").is_none());
}

#[test]
fn malformed_success_prefix_with_trailing_guest_output_is_rejected() {
    let (output, report) = run_with_adapters(
        "trailing-guest-output",
        r#"sh -c 'exit 0'"#,
        r#"sh -c 'printf "PARITY_OK|actor-system|42\nEXTRA\n"'"#,
    );
    assert!(!output.status.success());
    let actor = case(&report, "actor-system");
    assert_eq!(actor["compile"]["status"], "completed");
    assert_eq!(actor["run"]["status"], "completed");
    assert_eq!(actor["run"]["exit_code"], 0);
    assert_eq!(actor["status"], "wrong_stdout");
    assert!(report["summary"].get("pass").is_none());
}

#[test]
fn canonical_guest_output_is_still_exploratory_without_validation() {
    let (output, report) = run_with_adapters(
        "exploratory",
        r#"sh -c 'exit 0'"#,
        r#"sh -c 'printf "PARITY_OK|actor-system|15\n"'"#,
    );
    assert!(!output.status.success());
    let actor = case(&report, "actor-system");
    assert_eq!(actor["run"]["exit_code"], 0);
    assert_eq!(actor["status"], "exploratory_match_not_parity_pass");
    assert!(report["summary"].get("pass").is_none());
}

#[test]
fn infrastructure_exit_125_prevents_guest_execution() {
    let (output, report) = run_with_adapters(
        "infrastructure",
        r#"sh -c 'exit 125'"#,
        r#"sh -c 'printf "PARITY_OK|actor-system|42\n"'"#,
    );
    assert!(!output.status.success());
    assert_eq!(report["summary"]["compile_infrastructure_failure"], 54);
    for record in report["records"].as_array().unwrap() {
        assert!(record["run"].is_null());
        assert_eq!(record["status"], "compile_infrastructure_failure");
    }
}

#[test]
fn two_backend_output_disagreement_is_an_explicit_parity_failure() {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "ores-parity-two-backends-{}-{serial}.json",
        std::process::id()
    ));
    let mut command = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"));
    command.args([
        "--execute",
        "--allow-unvalidated",
        "--timeout",
        "2",
        "--output",
    ]);
    command.arg(&path);
    for backend in ["WASM", "DOTNET", "FORGE", "CRANELIFT", "JS"] {
        command.env_remove(format!("ORES_PARITY_{backend}_COMPILE"));
        command.env_remove(format!("ORES_PARITY_{backend}_RUN"));
    }
    command.env("ORES_PARITY_JAVA_COMPILE", r#"sh -c 'exit 0'"#);
    command.env("ORES_PARITY_LLVM_COMPILE", r#"sh -c 'exit 0'"#);
    command.env(
        "ORES_PARITY_JAVA_RUN",
        r#"sh -c 'printf "PARITY_OK|parameter-syntax|42\n"'"#,
    );
    command.env(
        "ORES_PARITY_LLVM_RUN",
        r#"sh -c 'printf "PARITY_OK|parameter-syntax|43\n"'"#,
    );
    let output = command
        .output()
        .expect("execute two synthetic backend adapters");
    let report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert_eq!(report["cases"], 54);
    assert_eq!(report["records"].as_array().unwrap().len(), 324);
    for backend in ["java", "llvm"] {
        let actor = report["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["case_id"] == "parameter-syntax" && record["backend"] == backend)
            .unwrap();
        assert_eq!(actor["status"], "cross_backend_stdout_mismatch");
        assert_eq!(actor["run"]["exit_code"], 0);
    }
    assert!(
        report["summary"]["cross_backend_stdout_mismatch"]
            .as_u64()
            .unwrap()
            >= 2
    );
    assert!(report["summary"].get("pass").is_none());
}

#[test]
fn plausible_but_wrong_actor_checksum_is_rejected_by_independent_oracle() {
    let (output, report) = run_with_adapters(
        "wrong-checksum",
        r#"sh -c 'exit 0'"#,
        r#"sh -c 'printf "PARITY_OK|actor-system|42\n"'"#,
    );
    assert!(!output.status.success());
    let actor = case(&report, "actor-system");
    assert_eq!(actor["run"]["exit_code"], 0);
    assert_eq!(actor["status"], "wrong_stdout");
    assert!(report["summary"].get("pass").is_none());
}
