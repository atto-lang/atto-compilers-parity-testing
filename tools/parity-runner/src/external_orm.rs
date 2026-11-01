//! Opt-in, exact-head real ORM source parity. No Git network, no private
//! source mirroring, no backend-specific code rewriting or synthetic guest.
use super::{fingerprint_graph, safe_file, sha256_file, TestCase};
use std::fs;
use std::path::Path;
use std::process::Command;

const FILES: [&str; 6] = [
    "src/schema.ores",
    "src/pg.ores",
    "tests/schema-plans.ores",
    "tests/schema-adversarial.ores",
    "tests/schema-invariants.ores",
    "tests/plans.ores",
];

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .arg("-C").arg(root).args(args).output()
        .map_err(|e| format!("ORM Git admission unavailable: {e}"))
}


// Check both module and guest bytes before compile and after guest completion.
// This guards source changes after initial Git admission; it does not make a
// race-free sandbox or provide any backend-specific compiler capability.
pub(super) fn still_bound(case: &TestCase) -> bool {
    if case.suite != "external-orm-exact-head" {
        return true;
    }
    let Some(root) = case.source.parent().and_then(|p| p.parent()) else {
        return false;
    };
    let Some(paths) = &case.graph_files else {
        return false;
    };
    let Some(expected) = &case.graph_sha256 else {
        return false;
    };
    let mut files = Vec::new();
    for path in paths {
        let original = root.join(path);
        let Ok(meta) = fs::symlink_metadata(&original) else {
            return false;
        };
        if !meta.file_type().is_file() || meta.file_type().is_symlink()
            || meta.len() == 0 || meta.len() > 1024 * 1024
        {
            return false;
        }
        let Ok(file) = safe_file(root, path) else {
            return false;
        };
        if file != original {
            return false;
        }
        files.push(file);
    }
    fingerprint_graph(&files, root)
        .is_ok_and(|actual| actual.as_str() == expected.as_str())
        && sha256_file(&case.source)
            .is_ok_and(|actual| actual.as_str() == case.source_sha256.as_str())
}


// Strict native artifact admission only for REAL external ORM sources. A
// compile wrapper that exits zero without emitting a bounded regular artifact
// is NOT a native compiler-backed ORM execution. JVM/Cranelift/JS runners are
// source-based and have separate guest requirements.
pub(super) fn needs_artifact(case: &TestCase, backend: &str) -> bool {
    case.suite == "external-orm-exact-head"
        && matches!(backend, "llvm" | "wasm" | "dotnet")
}

pub(super) fn adapter_admitted(
    case: &TestCase, backend: &str, compile: &str, run: &str
) -> bool {
    if case.suite != "external-orm-exact-head" { return true; }
    if !compile.contains("{source}") || run.is_empty() { return false; }
    match backend {
        "java" | "cranelift" | "js" => run.contains("{source}"),
        "wasm" => compile.contains("{artifact}")
            && run.contains("{artifact}")
            && run.contains("{artifact_sha256}"),
        // AOT binaries must execute directly; a shell wrapper that prints
        // a fabricated guest receipt cannot be the run phase.
        "llvm" | "dotnet" => compile.contains("{artifact}")
            && run.trim() == "{artifact}",
        _ => false,
    }
}

pub(super) fn compiled_artifact_digest(
    case: &TestCase, backend: &str, path: &Path
) -> Result<Option<String>, String> {
    if !needs_artifact(case, backend) { return Ok(None); }
    let meta = fs::symlink_metadata(path).map_err(|e| format!("native artifact absent: {e}"))?;
    if !meta.file_type().is_file() || meta.file_type().is_symlink()
        || meta.len() == 0 || meta.len() > 64 * 1024 * 1024
    {
        return Err("native artifact must be bounded nonempty regular file".into());
    }
    sha256_file(path).map(Some)
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
    // Reject a nested directory or a different enclosing repository that
    // happens to expose a matching HEAD. Source paths are relative to the
    // actual Git worktree root and must not be resolved by search paths.
    let git_top = git(&root, &["rev-parse", "--show-toplevel"])?;
    if !git_top.status.success()
        || Path::new(String::from_utf8_lossy(&git_top.stdout).trim()) != root.as_path()
    {
        return Err("ORM root must be the Git worktree top-level".into());
    }
    let rev = git(&root, &["rev-parse", "HEAD"])?;
    if !rev.status.success() || String::from_utf8_lossy(&rev.stdout).trim() != head {
        return Err("ORM checkout HEAD does not match --orm-head".into());
    }
    let tree = git(&root, &["rev-parse", "HEAD^{tree}"])?;
    if !tree.status.success() {
        return Err("ORM Git tree lookup failed".into());
    }
    let tree_id = String::from_utf8(tree.stdout)
        .map_err(|_| "ORM Git tree identity is not UTF-8")?
        .trim()
        .to_string();
    if tree_id.len() != 40 || !tree_id.bytes().all(|x| x.is_ascii_hexdigit()) {
        return Err("ORM tree identity must be a 40-hex Git SHA".into());
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
    for (index, shared) in validated.iter().take(2).enumerate() {
        let module = fs::read_to_string(shared).map_err(|e| e.to_string())?;
        if module.lines().any(|line| line.trim_start().starts_with("import ")) {
            return Err(format!("ORM source module dependency graph changed: {}", FILES[index]));
        }
    }
    let tests = [
        ("orm-schema-plans", 2usize, 0usize,
         "import module schema from '../src/schema';",
         "Pure Oreslang schema-diff planning contracts passed\n"),
        ("orm-schema-adversarial", 3, 0,
         "import module schema from '../src/schema';",
         "Pure Oreslang adversarial schema contracts passed\n"),
        ("orm-schema-invariants", 4, 0,
         "import module schema from '../src/schema';",
         "Pure Oreslang schema invariant and metamorphic contracts passed\n"),
        ("orm-sql-plans", 5, 1,
         "import module pg from '../src/pg';",
         "Postgres SQL planning contracts passed\n"),
    ];
    let mut cases = Vec::new();
    for (name, file_index, module_index, import, stdout) in tests {
        let entry = &validated[file_index];
        let code = fs::read_to_string(entry).map_err(|e| e.to_string())?;
        let imports: Vec<_> = code.lines()
            .filter(|line| line.trim_start().starts_with("import ")).collect();
        if imports.len() != 1 || imports[0].trim() != import {
            return Err(format!("ORM guest import graph changed: {}", FILES[file_index]));
        }
        let graph = vec![validated[module_index].clone(), entry.clone()];
        cases.push(TestCase {
            suite: "external-orm-exact-head".into(),
            case_id: name.into(),
            name: name.into(),
            source: entry.clone(),
            source_sha256: sha256_file(entry)?,
            graph_sha256: Some(fingerprint_graph(&graph, &root)?),
            graph_files: Some(vec![FILES[module_index].into(), FILES[file_index].into()]),
            expected_kind: "exit".into(),
            expected_value: Some(0),
            expected_diagnostic_category: None,
            expected_stdout_prefix: Some(stdout.into()),
            validation_status: "unvalidated".into(),
            features: vec!["pure-orm-source".into(), "schema-and-sql".into(),
                           format!("source-commit:{head}"),
                           format!("source-tree:{tree_id}")],
        });
    }
    Ok(cases)
}
