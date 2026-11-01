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
        assert!(case["features"].as_array().unwrap().iter()
            .any(|feature| feature == &format!("source-commit:{head}")));
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
