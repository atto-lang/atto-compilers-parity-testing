//! Opt-in, exact-head real ORM source parity. No Git network, no private
//! source mirroring, no backend-specific code rewriting or synthetic guest.
use super::{fingerprint_graph, safe_file, sha256_file, TestCase};
use std::fs;
use std::path::Path;
use std::process::Command;

const FILES: [&str; 4] = [
    "src/schema.ores",
    "tests/schema-plans.ores",
    "tests/schema-adversarial.ores",
    "tests/schema-invariants.ores",
];

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .arg("-C").arg(root).args(args).output()
        .map_err(|e| format!("ORM Git admission unavailable: {e}"))
}

pub(super) fn load(source_root: &Path, head: &str) -> Result<Vec<TestCase>, String> {
    if head.len() != 40 || !head.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        return Err("--orm-head must be a full lowercase 40-hex Git commit".into());
    }
    let root = source_root.canonicalize()
        .map_err(|e| format!("ORM checkout not accessible: {e}"))?;
    if !root.is_dir() {
        return Err("ORM checkout is not a directory".into());
    }
    let rev = git(&root, &["rev-parse", "HEAD"])?;
    if !rev.status.success() || String::from_utf8_lossy(&rev.stdout).trim() != head {
        return Err("ORM checkout HEAD does not match --orm-head".into());
    }
    let mut validated = Vec::new();
    for rel in FILES {
        let full = root.join(rel);
        let meta = fs::symlink_metadata(&full)
            .map_err(|e| format!("missing required ORM file {rel}: {e}"))?;
        if !meta.file_type().is_file() || meta.file_type().is_symlink()
            || meta.len() == 0 || meta.len() > 1024 * 1024
        {
            return Err(format!("ORM fixture must be a bounded regular file: {rel}"));
        }
        let path = safe_file(&root, rel)?;
        if path != full {
            return Err(format!("ORM fixture resolves to another path: {rel}"));
        }
        let tracked = git(&root, &["ls-files", "--error-unmatch", "--", rel])?;
        if !tracked.status.success() {
            return Err(format!("ORM fixture not committed at selected HEAD: {rel}"));
        }
        // Git status can be bypassed by assume-unchanged or skip-worktree.
        // Bind the actual bytes to the pinned HEAD's blob identity instead.
        let expected_blob = git(&root, &["rev-parse", &format!("HEAD:{rel}")])?;
        let observed_blob = git(&root, &["hash-object", "--", rel])?;
        if !expected_blob.status.success() || !observed_blob.status.success()
            || expected_blob.stdout != observed_blob.stdout
        {
            return Err(format!("ORM on-disk bytes differ from pinned Git blob: {rel}"));
        }
        let status = git(&root, &["status", "--porcelain=v1", "--untracked-files=all", "--", rel])?;
        if !status.status.success() || !status.stdout.is_empty() {
            return Err(format!("ORM fixture has staged, unstaged or untracked changes: {rel}"));
        }
        validated.push(path);
    }
    let shared = &validated[0];
    let module = fs::read_to_string(shared).map_err(|e| e.to_string())?;
    if module.lines().any(|line| line.trim_start().starts_with("import ")) {
        return Err("ORM schema module dependency graph changed; reviewer must re-admit it".into());
    }
    let tests = [
        ("orm-schema-plans", "Pure Oreslang schema-diff planning contracts passed\n"),
        ("orm-schema-adversarial", "Pure Oreslang adversarial schema contracts passed\n"),
        ("orm-schema-invariants", "Pure Oreslang schema invariant and metamorphic contracts passed\n"),
    ];
    let mut cases = Vec::new();
    for (index, (name, stdout)) in tests.iter().enumerate() {
        let entry = &validated[index + 1];
        let code = fs::read_to_string(entry).map_err(|e| e.to_string())?;
        let imports: Vec<_> = code.lines().filter(|line| line.trim_start().starts_with("import ")).collect();
        if imports.len() != 1 || imports[0].trim() != "import module schema from '../src/schema';" {
            return Err(format!("ORM guest import graph changed: {}", FILES[index + 1]));
        }
        let graph = vec![shared.clone(), entry.clone()];
        cases.push(TestCase {
            suite: "external-orm-exact-head".into(),
            case_id: (*name).into(),
            name: (*name).into(),
            source: entry.clone(),
            source_sha256: sha256_file(entry)?,
            graph_sha256: Some(fingerprint_graph(&graph, &root)?),
            graph_files: Some(vec![FILES[0].into(), FILES[index + 1].into()]),
            expected_kind: "exit".into(),
            expected_value: Some(0),
            expected_diagnostic_category: None,
            expected_stdout_prefix: Some((*stdout).into()),
            validation_status: "unvalidated".into(),
            features: vec!["pure-orm-source".into(), "schema-stable-id".into(),
                           format!("source-commit:{head}")],
        });
    }
    Ok(cases)
}
