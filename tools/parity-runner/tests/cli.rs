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
    assert_eq!(report["records"].as_array().unwrap().len(), 67);
    assert_eq!(report["summary"]["backend_not_configured"], 67);
}

#[cfg(unix)]
#[test]
fn adapter_error_is_infrastructure_failure_even_for_negative_probes() {
    let (status, report) = execute("adapter-error", true);
    assert!(!status.success());
    assert_eq!(report["summary"]["compile_infrastructure_failure"], 67);
}

#[test]
fn cranelift_is_explicit_and_unconfigured_is_a_failure() {
    let path = std::env::temp_dir().join(format!(
        "ores-cli-{}-cranelift-unconfigured.json",
        std::process::id()
    ));
    let result = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args([
            "--backend",
            "cranelift",
            "--execute",
            "--allow-unvalidated",
            "--output",
        ])
        .arg(&path)
        .env_remove("ORES_PARITY_CRANELIFT_COMPILE")
        .env_remove("ORES_PARITY_CRANELIFT_RUN")
        .status()
        .unwrap();
    assert!(!result.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(report["backends"], serde_json::json!(["cranelift"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 67);
    assert_eq!(report["summary"]["backend_not_configured"], 67);
}

#[test]
fn selected_scalar_case_is_one_nonexecuted_record() {
    let path = std::env::temp_dir().join(format!(
        "ores-cli-{}-cranelift-scalar-selection.json",
        std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--case", "01", "--output"])
        .arg(&path)
        .status()
        .unwrap();
    assert!(status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(report["cases"], 1);
    assert_eq!(report["corpus"]["total_cases"], 67);
    assert_eq!(report["records"].as_array().unwrap().len(), 1);
    assert_eq!(report["records"][0]["case_id"], "01");
    assert_eq!(report["records"][0]["backend"], "cranelift");
    assert_eq!(report["summary"]["not_run"], 1);
    assert_eq!(report["executed"], false);
}

#[test]
fn unknown_scalar_case_fails_without_faking_success() {
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--case", "NOT-A-REAL-CASE"])
        .status()
        .unwrap();
    assert!(!status.success());
}
