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
    assert_eq!(report["records"].as_array().unwrap().len(), 59);
    assert_eq!(report["summary"]["backend_not_configured"], 59);
}

#[cfg(unix)]
#[test]
fn adapter_error_is_infrastructure_failure_even_for_negative_probes() {
    let (status, report) = execute("adapter-error", true);
    assert!(!status.success());
    assert_eq!(report["summary"]["compile_infrastructure_failure"], 59);
}

#[test]
fn cranelift_missing_adapter_fails_closed() {
    let output = std::env::temp_dir().join(format!(
        "ores-cli-{}-cranelift.json", std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--execute", "--allow-unvalidated", "--output"])
        .arg(&output)
        .env_remove("ORES_PARITY_CRANELIFT_COMPILE")
        .env_remove("ORES_PARITY_CRANELIFT_RUN")
        .status()
        .unwrap();
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(!status.success());
    assert_eq!(report["backends"], serde_json::json!(["cranelift"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 59);
    assert_eq!(report["summary"]["backend_not_configured"], 59);
}

#[test]
fn case_filter_runs_only_requested_scalar_inventory() {
    let output = std::env::temp_dir().join(format!(
        "ores-cli-{}-cranelift-eight.json",
        std::process::id()
    ));
    let mut command = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"));
    command.args(["--backend", "cranelift", "--execute", "--allow-unvalidated"]);
    for id in ["34", "35", "36", "37", "38", "39", "40", "41"] {
        command.args(["--case", id]);
    }
    let status = command.args(["--output"]).arg(&output)
        .env_remove("ORES_PARITY_CRANELIFT_COMPILE")
        .env_remove("ORES_PARITY_CRANELIFT_RUN")
        .status()
        .unwrap();
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(!status.success());
    assert_eq!(report["corpus"]["total_cases"], 59);
    assert_eq!(report["cases"], 8);
    assert_eq!(report["records"].as_array().unwrap().len(), 8);
    assert_eq!(report["summary"]["backend_not_configured"], 8);
    assert!(report["records"].as_array().unwrap().iter().all(|record|
        record["backend"] == "cranelift" && record["status"] == "backend_not_configured"
    ));
}

#[test]
fn unknown_case_filter_is_error_without_a_report() {
    let output = std::env::temp_dir().join(format!(
        "ores-cli-{}-unknown-filter.json",
        std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--case", "nonexistent-case", "--output"])
        .arg(&output)
        .status()
        .unwrap();
    assert!(!status.success());
    assert!(!output.exists());
}
