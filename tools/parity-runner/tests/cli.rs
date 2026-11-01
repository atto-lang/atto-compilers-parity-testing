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
    assert_eq!(report["records"].as_array().unwrap().len(), 63);
    assert_eq!(report["summary"]["backend_not_configured"], 63);
}

#[cfg(unix)]
#[test]
fn adapter_error_is_infrastructure_failure_even_for_negative_probes() {
    let (status, report) = execute("adapter-error", true);
    assert!(!status.success());
    assert_eq!(report["summary"]["compile_infrastructure_failure"], 63);
}

#[test]
fn required_inventory_includes_forge_and_cranelift_but_not_optional_js() {
    let output =
        std::env::temp_dir().join(format!("ores-parity-required-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--output"])
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(
        report["backends"],
        serde_json::json!(["llvm", "java", "wasm", "dotnet", "forge", "cranelift"])
    );
    let manifest_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest_path).unwrap()).unwrap();
    assert_eq!(report["backends"], manifest["policy"]["target_backends"]);
    assert_eq!(report["corpus"]["legacy_cases"], 45);
    assert_eq!(report["cases"], 63);
    assert_eq!(report["records"].as_array().unwrap().len(), 378);
    assert_eq!(report["summary"]["not_run"], 378);
    let candidates: Vec<_> = report["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| ["32", "33", "34", "35"].contains(&r["case_id"].as_str().unwrap_or("")))
        .collect();
    assert_eq!(candidates.len(), 24);
    assert!(candidates
        .iter()
        .all(|r| r["validation_status"] == "candidate" && r["status"] == "not_run"));
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
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(report["backends"], serde_json::json!(["js"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 63);
    assert_eq!(report["summary"]["not_run"], 63);
}

#[test]
fn forge_diagnostic_rules_never_classify_generic_parse_errors_as_unknown_names() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../diagnostics.json");
    let rules: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(rules["forge"]["integer_range"][0], "invalid i64");
    assert_eq!(rules["forge"]["arity"][0], "function arity mismatch:");
    assert_eq!(
        rules["forge"]["duplicate_symbol"][0],
        "duplicate function at byte"
    );
    assert!(rules["forge"].get("unknown_identifier").is_none());
    assert!(rules["forge"].get("type_mismatch").is_none());
}

#[test]
fn missing_required_forge_adapter_is_a_hard_failure() {
    let output = std::env::temp_dir().join(format!(
        "ores-parity-forge-missing-{}.json",
        std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args([
            "--backend",
            "forge",
            "--execute",
            "--allow-unvalidated",
            "--output",
        ])
        .arg(&output)
        .env_remove("ORES_PARITY_FORGE_COMPILE")
        .env_remove("ORES_PARITY_FORGE_RUN")
        .status()
        .unwrap();
    assert!(!status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(report["backends"], serde_json::json!(["forge"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 63);
    assert_eq!(report["summary"]["backend_not_configured"], 63);
}

#[test]
fn cranelift_missing_adapter_fails_instead_of_fabricating_parity() {
    let output = std::env::temp_dir()
        .join(format!("ores-parity-cranelift-missing-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--execute", "--allow-unvalidated", "--output"])
        .arg(&output)
        .env_remove("ORES_PARITY_CRANELIFT_COMPILE")
        .env_remove("ORES_PARITY_CRANELIFT_RUN")
        .status()
        .unwrap();
    assert!(!status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(report["backends"], serde_json::json!(["cranelift"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 63);
    assert_eq!(report["summary"]["backend_not_configured"], 63);
}

#[test]
fn new_i64_candidates_remain_not_run_even_with_sha256_fingerprints() {
    let output = std::env::temp_dir()
        .join(format!("ores-parity-cranelift-candidates-{}.json", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .args(["--backend", "cranelift", "--output"])
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    let new_cases: Vec<_> = report["records"].as_array().unwrap().iter()
        .filter(|item| item["case_id"].as_str().and_then(|s| s.parse::<u8>().ok())
            .is_some_and(|id| (40..=49).contains(&id)))
        .collect();
    assert_eq!(new_cases.len(), 10);
    assert!(new_cases.iter().all(|item| item["status"] == "not_run"
        && item["validation_status"] == "candidate"
        && item["source_sha256"].as_str().is_some_and(|s| s.len() == 64)));
}
