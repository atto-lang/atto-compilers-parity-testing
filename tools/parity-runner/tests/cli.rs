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
fn required_inventory_includes_forge_but_not_optional_js() {
    let output = std::env::temp_dir().join(format!("ores-parity-required-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--output"])
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(report["backends"], serde_json::json!(["llvm", "java", "wasm", "dotnet", "forge"]));
    assert_eq!(report["corpus"]["legacy_cases"], 35);
    assert_eq!(report["cases"], 53);
    assert_eq!(report["records"].as_array().unwrap().len(), 265);
    assert_eq!(report["summary"]["not_run"], 265);
    let candidates: Vec<_> = report["records"].as_array().unwrap().iter()
        .filter(|r| ["32", "33", "34", "35"].contains(&r["case_id"].as_str().unwrap_or("")))
        .collect();
    assert_eq!(candidates.len(), 20);
    assert!(candidates.iter().all(|r| r["validation_status"] == "candidate" && r["status"] == "not_run"));
}

#[test]
fn optional_js_requires_explicit_selection() {
    let output = std::env::temp_dir().join(format!("ores-parity-js-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "js", "--output"])
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(report["backends"], serde_json::json!(["js"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 53);
    assert_eq!(report["summary"]["not_run"], 53);
}
