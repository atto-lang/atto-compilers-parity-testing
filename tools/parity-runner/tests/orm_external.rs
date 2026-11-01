// Host-only test: mock checkout never claims guest compiler execution.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> String {
    let o = Command::new("git").arg("-C").arg(root).args(args)
        .output().expect("git required");
    assert!(o.status.success(), "git failed: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap().trim().to_string()
}

fn fake_checkout() -> (PathBuf, String) {
    let root = std::env::temp_dir().join(format!(
        "ores-orm-admission-{}-{:?}", std::process::id(), std::thread::current().id()));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    git(&root, &["init", "-q"]);
    for path in ["src/schema.ores", "src/pg.ores"] {
        fs::write(root.join(path), "pub fnc fake(): int { return 42; }\n").unwrap();
    }
    for path in ["schema-plans.ores", "schema-adversarial.ores", "schema-invariants.ores"] {
        fs::write(root.join("tests").join(path),
            "import module schema from '../src/schema';\npub fnc main(): void { return; }\n").unwrap();
    }
    fs::write(root.join("tests/plans.ores"),
        "import module pg from '../src/pg';\npub fnc main(): void { return; }\n").unwrap();
    git(&root, &["add", "src", "tests"]);
    git(&root, &["-c", "user.email=fixture@example.test",
        "-c", "user.name=Fixture", "commit", "-qm", "fixture"]);
    let head = git(&root, &["rev-parse", "HEAD"]);
    (root, head)
}

fn run(root: &Path, head: &str) -> (bool, serde_json::Value) {
    let out = root.join("test-report.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .arg("--orm-root").arg(root).arg("--orm-head").arg(head)
        .args(["--backend", "cranelift", "--case-id", "orm-schema-plans",
               "--case-id", "orm-schema-adversarial",
               "--case-id", "orm-schema-invariants",
               "--case-id", "orm-sql-plans", "--output"])
        .arg(&out).output().unwrap();
    let report = if out.exists() {
        serde_json::from_slice(&fs::read(out).unwrap()).unwrap()
    } else {
        serde_json::Value::Null
    };
    (output.status.success(), report)
}

#[test]
fn four_real_orm_source_candidates_are_not_compiler_evidence() {
    let (root, head) = fake_checkout();
    let (ok, r) = run(&root, &head);
    assert!(ok);
    assert_eq!(r["cases"], 59);
    assert_eq!(r["corpus"]["total_cases"], 59);
    assert_eq!(r["corpus"]["external_orm_cases"], 4);
    assert_eq!(r["executed"], false);
    assert_eq!(r["summary"]["not_run"], 4);
    assert_eq!(r["records"].as_array().unwrap().len(), 4);
    for case in r["records"].as_array().unwrap() {
        assert_eq!(case["backend"], "cranelift");
        assert_eq!(case["suite"], "external-orm-exact-head");
        assert_eq!(case["graph_sha256"].as_str().unwrap().len(), 64);
        let sha = format!("source-commit:{head}");
        let tree = format!("source-tree:{}", git(&root, &["rev-parse", "HEAD^{tree}"]));
        let feature_list = case["features"].as_array().unwrap();
        assert!(feature_list.iter().any(|feature| feature.as_str() == Some(sha.as_str())));
        assert!(feature_list.iter().any(|feature| feature.as_str() == Some(tree.as_str())));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mutated_or_disappearing_orm_source_fails_closed() {
    let (root, head) = fake_checkout();
    fs::write(root.join("src/schema.ores"), "pub fnc changed(): int { return 7; }\n").unwrap();
    assert!(!run(&root, &head).0);
    // Git status alone may skip assume-unchanged. Object comparison must
    // independently bind the exact bytes to HEAD.
    git(&root, &["update-index", "--assume-unchanged", "src/schema.ores"]);
    assert!(!run(&root, &head).0);
    fs::remove_dir_all(root).unwrap();

    let (root, head) = fake_checkout();
    fs::remove_file(root.join("tests/plans.ores")).unwrap();
    assert!(!run(&root, &head).0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn committed_import_graph_drift_requires_new_review() {
    let (root, _) = fake_checkout();
    fs::write(root.join("tests/plans.ores"),
        "import module pg from '../src/unknown';\npub fnc main(): void { return; }\n").unwrap();
    git(&root, &["add", "tests/plans.ores"]);
    git(&root, &["-c", "user.email=fixture@example.test",
        "-c", "user.name=Fixture", "commit", "-qm", "drift"]);
    let head = git(&root, &["rev-parse", "HEAD"]);
    assert!(!run(&root, &head).0);
    fs::remove_dir_all(root).unwrap();
}


#[test]
fn all_six_backends_receive_identical_real_orm_guest_graph_fingerprints() {
    // This tests inventory/provenance only, NOT actual Oreslang compilation.
    let (root, head) = fake_checkout();
    let out = root.join("six-backend-inventory.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .arg("--orm-root").arg(&root)
        .arg("--orm-head").arg(&head)
        .args([
            "--case-id", "orm-schema-plans",
            "--case-id", "orm-schema-adversarial",
            "--case-id", "orm-schema-invariants",
            "--case-id", "orm-sql-plans",
            "--output",
        ])
        .arg(&out).output().unwrap();
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(out).unwrap()).unwrap();
    assert_eq!(report["executed"], false);
    assert_eq!(report["records"].as_array().unwrap().len(), 24);
    assert_eq!(report["summary"]["not_run"], 24);
    let mut by_case = std::collections::BTreeMap::new();
    for cell in report["records"].as_array().unwrap() {
        assert_eq!(cell["suite"], "external-orm-exact-head");
        assert_eq!(cell["status"], "not_run");
        let name = cell["case_id"].as_str().unwrap().to_string();
        let identity = (
            cell["source_sha256"].as_str().unwrap().to_string(),
            cell["graph_sha256"].as_str().unwrap().to_string(),
        );
        let entry = by_case.entry(name).or_insert_with(|| identity.clone());
        assert_eq!(*entry, identity, "backend-specific ORM source was substituted");
    }
    assert_eq!(by_case.len(), 4);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_real_backend_compiler_cannot_be_reported_as_orm_parity() {
    let (root, head) = fake_checkout();
    let out = root.join("no-compiler.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .arg("--orm-root").arg(&root)
        .arg("--orm-head").arg(&head)
        .args([
            "--backend", "cranelift",
            "--case-id", "orm-sql-plans",
            "--execute", "--allow-unvalidated",
            "--output",
        ])
        .arg(&out)
        .env_remove("ORES_PARITY_CRANELIFT_COMPILE")
        .env_remove("ORES_PARITY_CRANELIFT_RUN")
        .output().unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(out).unwrap()).unwrap();
    assert_eq!(report["records"].as_array().unwrap().len(), 1);
    assert_eq!(report["summary"]["backend_not_configured"], 1);
    assert_eq!(report["records"][0]["status"], "backend_not_configured");
    assert_eq!(report["executed"], true);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn wrong_exact_head_never_admits_actual_orm_source() {
    let (root, _) = fake_checkout();
    let (accepted, report) = run(&root, "0000000000000000000000000000000000000000");
    assert!(!accepted);
    assert!(report.is_null());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nested_worktree_path_never_counts_as_separately_pinned_orm_checkout() {
    let (root, head) = fake_checkout();
    let nested = root.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .arg("--orm-root").arg(nested)
        .arg("--orm-head").arg(head)
        .output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("ORM root must be the Git worktree top-level"));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn real_java_llvm_lane_has_valid_shell_and_rejects_missing_pins() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/run-orm-java-llvm.sh");
    let syntax = Command::new("bash").arg("-n").arg(&path).output().unwrap();
    assert!(syntax.status.success(), "bash syntax: {}",
        String::from_utf8_lossy(&syntax.stderr));
    let missing_args = Command::new("bash").arg(path).output().unwrap();
    assert!(!missing_args.status.success());
    assert!(missing_args.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing_args.stderr)
        .contains("PRECHECK_FAILED"));
}


#[cfg(unix)]
#[test]
fn fake_compiler_success_without_native_artifact_cannot_certify_any_aot_orm() {
    // Compile phase exits zero but produces NO executable. Source/test code
    // stays unchanged, and even matching fake stdout must never be a pass.
    for backend in ["llvm", "wasm", "dotnet"] {
        let (root, head) = fake_checkout();
        let report_path = root.join(format!("no-artifact-{backend}.json"));
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"));
        cmd.arg("--orm-root").arg(&root).arg("--orm-head").arg(&head)
            .args(["--backend", backend, "--case-id", "orm-sql-plans",
                   "--execute", "--allow-unvalidated", "--output"])
            .arg(&report_path);
        for key in ["LLVM","WASM","DOTNET"] {
            cmd.env_remove(format!("ORES_PARITY_{key}_COMPILE"));
            cmd.env_remove(format!("ORES_PARITY_{key}_RUN"));
        }
        let compile = "sh -c 'exit 0' -- {source} {artifact}";
        let run = if backend == "wasm" {
            "sh -c 'echo Postgres SQL planning contracts passed' -- {artifact} {artifact_sha256}"
        } else {
            "sh -c 'echo Postgres SQL planning contracts passed' -- {artifact}"
        };
        let env = backend.to_ascii_uppercase();
        let output = cmd.env(format!("ORES_PARITY_{env}_COMPILE"), compile)
            .env(format!("ORES_PARITY_{env}_RUN"), run)
            .output().unwrap();
        assert!(!output.status.success(), "{backend}: fake AOT compiler admitted");
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
        assert_eq!(report["summary"]["runtime_infrastructure_failure"], 1,
            "{backend} did not reject absent artifact");
        assert!(report["records"][0]["artifact_sha256"].is_null());
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn missing_wasm_digest_placeholder_is_a_hard_adapter_failure() {
    let (root, head) = fake_checkout();
    let out = root.join("unbound-wasm.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ores-parity-runner"))
        .arg("--orm-root").arg(&root).arg("--orm-head").arg(&head)
        .args(["--backend", "wasm", "--case-id", "orm-schema-plans",
               "--execute", "--allow-unvalidated", "--output"])
        .arg(&out)
        .env("ORES_PARITY_WASM_COMPILE", "sh -c 'exit 0' -- {source} {artifact}")
        .env("ORES_PARITY_WASM_RUN", "sh -c 'exit 0' -- {artifact}")
        .output().unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(out).unwrap()).unwrap();
    assert_eq!(report["summary"]["runtime_infrastructure_failure"], 1);
    fs::remove_dir_all(root).unwrap();
}
