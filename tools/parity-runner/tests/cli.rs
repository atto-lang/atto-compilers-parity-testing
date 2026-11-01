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
    assert_eq!(report["records"].as_array().unwrap().len(), 55);
    assert_eq!(report["summary"]["backend_not_configured"], 55);
}

#[cfg(unix)]
#[test]
fn adapter_error_is_infrastructure_failure_even_for_negative_probes() {
    let (status, report) = execute("adapter-error", true);
    assert!(!status.success());
    assert_eq!(report["summary"]["compile_infrastructure_failure"], 55);
}

#[test]
fn cranelift_portable_scalar_inventory_is_six_distinct_unexecuted_cases() {
    let output = std::env::temp_dir().join(format!(
        "ores-cli-{}-cranelift-scalar.json",
        std::process::id()
    ));
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"));
    cmd.args(["--backend", "cranelift"]);
    for id in ["32", "33", "34", "35", "36", "37"] {
        cmd.args(["--case-id", id]);
    }
    let status = cmd.args(["--output"]).arg(&output).status().unwrap();
    assert!(status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(report["backends"], serde_json::json!(["cranelift"]));
    assert_eq!(report["records"].as_array().unwrap().len(), 6);
    assert_eq!(report["summary"]["not_run"], 6);
    assert_eq!(report["executed"], false);
}

#[test]
fn unknown_and_duplicate_case_filters_fail_closed() {
    for ids in [&["not-a-case"][..], &["32", "32"][..]] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"));
        cmd.args(["--backend", "cranelift"]);
        for id in ids {
            cmd.args(["--case-id", id]);
        }
        assert!(!cmd.status().unwrap().success());
    }
}
