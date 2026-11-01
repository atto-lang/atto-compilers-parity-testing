use std::process::Command;

fn execute(name: &str, configured: bool) -> (std::process::ExitStatus, serde_json::Value) {
    let output = std::env::temp_dir().join(format!("ores-cli-{}-{name}.json", std::process::id()));
    let mut command = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"));
    command.args(["--backend", "java", "--output"]).arg(&output);
    if configured {
        command.args(["--execute", "--allow-unvalidated"]);
        command.env("ORES_PARITY_JAVA_COMPILE", "sh -c 'exit 125'");
        command.env("ORES_PARITY_JAVA_RUN", "sh -c 'exit 0'");
    } else {
        command.args(["--execute", "--allow-unvalidated"]);
        command.env_remove("ORES_PARITY_JAVA_COMPILE");
        command.env_remove("ORES_PARITY_JAVA_RUN");
    }
    let status = command.status().unwrap();
    let report = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    (status, report)
}

#[test]
fn selected_missing_backend_fails_instead_of_claiming_success() {
    let (status, report) = execute("missing", false);
    assert!(!status.success());
    assert_eq!(report["backends"], serde_json::json!(["java"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 49);
    assert_eq!(report["summary"]["backend_not_configured"], 49);
}

#[cfg(unix)]
#[test]
fn adapter_error_is_infrastructure_failure_even_for_negative_probes() {
    let (status, report) = execute("adapter-error", true);
    assert!(!status.success());
    assert_eq!(report["summary"]["compile_infrastructure_failure"], 49);
}

#[test]
fn cranelift_is_not_configured_instead_of_skipped_or_passed() {
    let output = std::env::temp_dir().join(format!("ores-cranelift-none-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--execute", "--allow-unvalidated", "--output"])
        .arg(&output)
        .env_remove("ORES_PARITY_CRANELIFT_COMPILE")
        .env_remove("ORES_PARITY_CRANELIFT_RUN")
        .status().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(!status.success());
    assert_eq!(report["backends"], serde_json::json!(["cranelift"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 49);
    assert_eq!(report["summary"]["backend_not_configured"], 49);
}

#[test]
fn cranelift_compile_exit_125_never_masquerades_as_parity() {
    let output = std::env::temp_dir().join(format!("ores-cranelift-125-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--execute", "--allow-unvalidated", "--output"])
        .arg(&output)
        .env("ORES_PARITY_CRANELIFT_COMPILE", "sh -c 'exit 125'")
        .env("ORES_PARITY_CRANELIFT_RUN", "sh -c 'exit 0'")
        .status().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(!status.success());
    assert_eq!(report["backends"], serde_json::json!(["cranelift"]));
    assert_eq!(report["summary"]["compile_infrastructure_failure"], 49);
    assert!(report["summary"].get("pass").is_none());
}

#[cfg(unix)]
#[test]
fn rejected_source_must_leave_no_compiler_artifacts() {
    let output = std::env::temp_dir().join(format!("ores-cranelift-negative-artifact-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--execute", "--allow-unvalidated", "--output"])
        .arg(&output)
        .env("ORES_PARITY_CRANELIFT_COMPILE", "sh -c 'touch {artifact}; exit 1'")
        .env("ORES_PARITY_CRANELIFT_RUN", "sh -c 'exit 0'")
        .status().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(!status.success());
    assert_eq!(report["records"].as_array().unwrap().len(), 49);
    // All negative compiler cases must be rejected independently of any
    // diagnostic that a malicious adapter might print.
    assert_eq!(report["summary"]["rejected_with_artifacts"], 8);
    assert!(report["summary"].get("pass").is_none());
}

#[cfg(unix)]
#[test]
fn compiler_diagnostic_cannot_be_spoofed_on_stdout() {
    let output = std::env::temp_dir().join(format!("ores-java-stdout-spoof-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "java", "--execute", "--allow-unvalidated", "--output"])
        .arg(&output)
        .env("ORES_PARITY_JAVA_COMPILE", "sh -c 'echo integer literal overflows signed 64-bit:; exit 1'")
        .env("ORES_PARITY_JAVA_RUN", "sh -c 'exit 42'")
        .status().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(!status.success());
    let range = report["records"].as_array().unwrap().iter()
        .find(|record| record["case_id"] == "13").unwrap();
    assert_eq!(range["status"], "rejection_unclassified");
    assert_eq!(range["actual_diagnostic_category"], serde_json::Value::Null);
}

#[cfg(unix)]
#[test]
fn matching_exploratory_output_never_returns_success() {
    let output = std::env::temp_dir().join(format!("ores-java-exploratory-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "java", "--execute", "--allow-unvalidated", "--output"])
        .arg(&output)
        .env("ORES_PARITY_JAVA_COMPILE", "sh -c 'exit 0'")
        .env("ORES_PARITY_JAVA_RUN", "sh -c 'exit 42'")
        .status().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(!status.success());
    let positive = report["records"].as_array().unwrap().iter()
        .find(|record| record["case_id"] == "01").unwrap();
    assert_eq!(positive["status"], "exploratory_match_not_parity_pass");
    assert!(report["summary"].get("pass").is_none());
}
