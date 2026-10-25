use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BACKENDS: [(&str, &str); 5] = [
    ("llvm", "LLVM"),
    ("java", "JAVA"),
    ("wasm", "WASM"),
    ("dotnet", "DOTNET"),
    ("js", "JS"),
];

#[derive(Debug, Deserialize)]
struct LegacyManifest {
    cases: Vec<LegacyCase>,
}

#[derive(Debug, Deserialize)]
struct LegacyCase {
    id: String,
    name: String,
    path: String,
    expected_kind: String,
    expected_value: Option<i32>,
    validation_status: String,
    features: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChecksManifest {
    checks: Vec<CheckRef>,
}

#[derive(Debug, Deserialize)]
struct CheckRef {
    name: String,
    expectation: String,
}

#[derive(Debug, Deserialize)]
struct Expectation {
    name: String,
    entry: String,
    stable_stdout_prefix: String,
    imported_files: usize,
    maturity: String,
    #[serde(default)]
    features: Vec<String>,
    verified: bool,
    #[serde(default)]
    negative_cases: Vec<NegativeExpectation>,
}

#[derive(Debug, Deserialize)]
struct NegativeExpectation {
    path: String,
    #[allow(dead_code)]
    reason: Option<String>,
}

#[derive(Debug)]
struct TestCase {
    suite: String,
    case_id: String,
    name: String,
    source: PathBuf,
    source_sha256: String,
    graph_sha256: Option<String>,
    graph_files: Option<Vec<String>>,
    expected_kind: String,
    expected_value: Option<i32>,
    expected_stdout_prefix: Option<String>,
    validation_status: String,
    features: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Invocation {
    status: String,
    exit_code: Option<i32>,
    stdout_hex: String,
    stderr_hex: String,
    duration_ms: u128,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct Record {
    suite: String,
    case_id: String,
    name: String,
    backend: String,
    source_sha256: String,
    graph_sha256: Option<String>,
    graph_files: Option<Vec<String>>,
    validation_status: String,
    features: Vec<String>,
    status: String,
    compile: Option<Invocation>,
    run: Option<Invocation>,
}

#[derive(Debug, Serialize)]
struct Corpus {
    legacy_cases: usize,
    multifile_checks: usize,
    negative_probes: usize,
    total_cases: usize,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: u32,
    executed: bool,
    corpus: Corpus,
    cases: usize,
    backends: Vec<String>,
    records: Vec<Record>,
    summary: BTreeMap<String, usize>,
}

fn hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(H[(b >> 4) as usize] as char);
        out.push(H[(b & 0x0f) as usize] as char);
    }
    out
}

fn parse_sha256_output(output: std::process::Output) -> Result<String, String> {
    if !output.status.success() {
        return Err(format!("sha256sum failed with {:?}", output.status.code()));
    }
    let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    text.split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| "sha256sum produced no digest".to_string())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| format!("failed to run sha256sum: {e}"))?;
    parse_sha256_output(output)
}

fn sha256_bytes(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run sha256sum: {e}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "sha256sum stdin was not piped".to_string())?;
    stdin.write_all(bytes).map_err(|e| e.to_string())?;
    drop(stdin);
    parse_sha256_output(child.wait_with_output().map_err(|e| e.to_string())?)
}

fn safe_file(root: &Path, path_text: &str) -> Result<PathBuf, String> {
    let path = root
        .join(path_text)
        .canonicalize()
        .map_err(|e| format!("{path_text}: {e}"))?;
    if !path.starts_with(root) || !path.is_file() {
        return Err(format!("invalid source path: {path_text}"));
    }
    Ok(path)
}

fn import_target(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("import ") {
        return None;
    }
    let marker = " from \"";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn local_imports(path: &Path, root: &Path) -> Result<Vec<PathBuf>, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for target in text.lines().filter_map(import_target) {
        if !target.starts_with('.') {
            continue;
        }
        let mut candidate = path
            .parent()
            .ok_or_else(|| format!("source has no parent: {}", path.display()))?
            .join(&target);
        if candidate.extension().is_none() {
            candidate.set_extension("ores");
        }
        let candidate = candidate
            .canonicalize()
            .map_err(|e| format!("{} imports {target}: {e}", path.display()))?;
        if !candidate.starts_with(root) || !candidate.is_file() {
            return Err(format!(
                "invalid local import from {}: {target}",
                path.display()
            ));
        }
        result.push(candidate);
    }
    Ok(result)
}

fn reachable_graph(entry: &Path, root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![entry.to_path_buf()];
    let mut seen = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        pending.extend(local_imports(&current, root)?);
    }
    Ok(seen.into_iter().collect())
}

fn relative_path(path: &Path, root: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|e| e.to_string())
}

fn fingerprint_graph(paths: &[PathBuf], root: &Path) -> Result<String, String> {
    let mut payload = Vec::new();
    for path in paths {
        let rel = relative_path(path, root)?;
        payload.extend_from_slice(rel.as_bytes());
        payload.push(0);
        payload.extend_from_slice(sha256_file(path)?.as_bytes());
        payload.push(b'\n');
    }
    sha256_bytes(&payload)
}

fn lowercase_kebab(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn load_cases(root: &Path) -> Result<(Vec<TestCase>, Corpus), String> {
    let legacy: LegacyManifest = serde_json::from_str(
        &fs::read_to_string(root.join("manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let mut cases = Vec::new();
    for case in &legacy.cases {
        let source = safe_file(root, &case.path)?;
        cases.push(TestCase {
            suite: "legacy".into(),
            case_id: case.id.clone(),
            name: case.name.clone(),
            source_sha256: sha256_file(&source)?,
            source,
            graph_sha256: None,
            graph_files: None,
            expected_kind: case.expected_kind.clone(),
            expected_value: case.expected_value,
            expected_stdout_prefix: None,
            validation_status: case.validation_status.clone(),
            features: case.features.clone(),
        });
    }

    let checks_manifest: ChecksManifest = serde_json::from_str(
        &fs::read_to_string(root.join("checks/manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut manifest_names = BTreeSet::new();
    let mut negative_probes = 0usize;

    for item in &checks_manifest.checks {
        if !manifest_names.insert(item.name.clone()) {
            return Err(format!("duplicate check name: {}", item.name));
        }
        if !lowercase_kebab(&item.name) {
            return Err(format!(
                "check name must be lowercase kebab-case: {}",
                item.name
            ));
        }
        let expectation_path = safe_file(root, &item.expectation)?;
        let expectation: Expectation = serde_json::from_str(
            &fs::read_to_string(&expectation_path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("{}: {e}", item.expectation))?;
        if expectation.name != item.name {
            return Err(format!("check name mismatch: {}", item.name));
        }

        let check_dir = root
            .join("checks")
            .join(&item.name)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let entry = safe_file(root, &expectation.entry)?;
        if entry.parent() != Some(check_dir.as_path()) {
            return Err(format!(
                "entry is outside its check directory: {}",
                expectation.entry
            ));
        }

        let direct_imports: BTreeSet<PathBuf> =
            local_imports(&entry, root)?.into_iter().collect();
        if direct_imports.len() != expectation.imported_files {
            return Err(format!(
                "{}: main imports {} local files, expected {}",
                item.name,
                direct_imports.len(),
                expectation.imported_files
            ));
        }
        let graph = reachable_graph(&entry, root)?;
        if graph.len() < 4 {
            return Err(format!(
                "{}: reachable source graph must contain at least four files",
                item.name
            ));
        }
        let graph_files = graph
            .iter()
            .map(|p| relative_path(p, root))
            .collect::<Result<Vec<_>, _>>()?;
        let graph_sha256 = fingerprint_graph(&graph, root)?;
        cases.push(TestCase {
            suite: "checks".into(),
            case_id: item.name.clone(),
            name: item.name.clone(),
            source_sha256: sha256_file(&entry)?,
            source: entry,
            graph_sha256: Some(graph_sha256),
            graph_files: Some(graph_files),
            expected_kind: "exit".into(),
            expected_value: Some(0),
            expected_stdout_prefix: Some(expectation.stable_stdout_prefix.clone()),
            validation_status: if expectation.verified {
                "validated".into()
            } else {
                "unvalidated".into()
            },
            features: expectation.features.clone(),
        });

        for (index, negative) in expectation.negative_cases.iter().enumerate() {
            let source = safe_file(root, &negative.path)?;
            if source.parent() != Some(check_dir.as_path()) {
                return Err(format!(
                    "negative probe is outside its check directory: {}",
                    negative.path
                ));
            }
            negative_probes += 1;
            let mut negative_features = expectation.features.clone();
            negative_features.push("negative-probe".into());
            cases.push(TestCase {
                suite: "checks".into(),
                case_id: format!("{}/negative-{}", item.name, index + 1),
                name: format!("{}-negative-{}", item.name, index + 1),
                source_sha256: sha256_file(&source)?,
                source,
                graph_sha256: None,
                graph_files: None,
                expected_kind: "compile_reject".into(),
                expected_value: None,
                expected_stdout_prefix: None,
                validation_status: "unvalidated".into(),
                features: negative_features,
            });
        }
    }

    let mut actual_names = BTreeSet::new();
    for entry in fs::read_dir(root.join("checks")).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        if entry.path().join("expectation.json").is_file() {
            actual_names.insert(entry.file_name().to_string_lossy().to_string());
        }
    }
    if actual_names != manifest_names {
        return Err(
            "checks/manifest.json does not exactly enumerate expectation-bearing check directories"
                .into(),
        );
    }

    let corpus = Corpus {
        legacy_cases: legacy.cases.len(),
        multifile_checks: checks_manifest.checks.len(),
        negative_probes,
        total_cases: cases.len(),
    };
    Ok((cases, corpus))
}

fn expand(
    template: &str,
    source: &Path,
    artifact: &Path,
    workdir: &Path,
) -> Result<Vec<String>, String> {
    let parts = shell_words::split(template).map_err(|e| e.to_string())?;
    Ok(parts
        .into_iter()
        .map(|p| {
            p.replace("{source}", &source.to_string_lossy())
                .replace("{artifact}", &artifact.to_string_lossy())
                .replace("{workdir}", &workdir.to_string_lossy())
        })
        .collect())
}

fn invoke(argv: &[String], workdir: &Path, timeout: Duration) -> Invocation {
    if argv.is_empty() {
        return Invocation {
            status: "infrastructure_error".into(),
            exit_code: None,
            stdout_hex: String::new(),
            stderr_hex: String::new(),
            duration_ms: 0,
            error: Some("empty command".into()),
        };
    }
    let start = Instant::now();
    let mut child = match Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return Invocation {
                status: "infrastructure_error".into(),
                exit_code: None,
                stdout_hex: String::new(),
                stderr_hex: String::new(),
                duration_ms: start.elapsed().as_millis(),
                error: Some(error.to_string()),
            }
        }
    };
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Invocation {
                    status: "timeout".into(),
                    exit_code: None,
                    stdout_hex: String::new(),
                    stderr_hex: String::new(),
                    duration_ms: start.elapsed().as_millis(),
                    error: None,
                };
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                return Invocation {
                    status: "infrastructure_error".into(),
                    exit_code: None,
                    stdout_hex: String::new(),
                    stderr_hex: String::new(),
                    duration_ms: start.elapsed().as_millis(),
                    error: Some(error.to_string()),
                }
            }
        }
    }
    match child.wait_with_output() {
        Ok(output) => Invocation {
            status: "completed".into(),
            exit_code: output.status.code(),
            stdout_hex: hex(&output.stdout),
            stderr_hex: hex(&output.stderr),
            duration_ms: start.elapsed().as_millis(),
            error: None,
        },
        Err(error) => Invocation {
            status: "infrastructure_error".into(),
            exit_code: None,
            stdout_hex: String::new(),
            stderr_hex: String::new(),
            duration_ms: start.elapsed().as_millis(),
            error: Some(error.to_string()),
        },
    }
}

fn safe_temp_component(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn temp_dir(case_id: &str, backend: &str) -> Result<PathBuf, String> {
    let path = env::temp_dir().join(format!(
        "ores-parity-{}-{}-{}",
        std::process::id(),
        safe_temp_component(case_id),
        safe_temp_component(backend)
    ));
    if path.exists() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

fn evaluate_case(
    case: &TestCase,
    backend: &str,
    env_name: &str,
    execute: bool,
    allow_unvalidated: bool,
    timeout: Duration,
) -> Result<Record, String> {
    let mut record = Record {
        suite: case.suite.clone(),
        case_id: case.case_id.clone(),
        name: case.name.clone(),
        backend: backend.into(),
        source_sha256: case.source_sha256.clone(),
        graph_sha256: case.graph_sha256.clone(),
        graph_files: case.graph_files.clone(),
        validation_status: case.validation_status.clone(),
        features: case.features.clone(),
        status: "not_run".into(),
        compile: None,
        run: None,
    };
    if !execute {
        return Ok(record);
    }
    if case.validation_status != "validated" && !allow_unvalidated {
        record.status = "unvalidated".into();
        return Ok(record);
    }

    let prefix = format!("ORES_PARITY_{env_name}");
    let compile_template = env::var(format!("{prefix}_COMPILE")).ok();
    let run_template = env::var(format!("{prefix}_RUN")).ok();
    if compile_template.is_none() || (case.expected_kind == "exit" && run_template.is_none()) {
        record.status = "backend_not_configured".into();
        return Ok(record);
    }

    let workdir = temp_dir(&case.case_id, backend)?;
    let artifact = workdir.join("program");
    let compile_argv = expand(
        compile_template.as_ref().expect("checked above"),
        &case.source,
        &artifact,
        &workdir,
    )?;
    let compile = invoke(&compile_argv, &workdir, timeout);
    let compile_status = compile.status.clone();
    let compile_exit = compile.exit_code;
    record.compile = Some(compile);

    if compile_status != "completed" {
        record.status = "compile_infrastructure_failure".into();
    } else if case.expected_kind == "compile_reject" {
        record.status = if compile_exit == Some(0) {
            "unexpected_compile_success".into()
        } else {
            "rejection_unclassified".into()
        };
    } else if compile_exit != Some(0) {
        record.status = "compile_failure".into();
    } else {
        let run_argv = expand(
            run_template.as_ref().expect("checked above"),
            &case.source,
            &artifact,
            &workdir,
        )?;
        let run = invoke(&run_argv, &workdir, timeout);
        let run_status = run.status.clone();
        let run_exit = run.exit_code;
        let stdout_hex = run.stdout_hex.clone();
        record.run = Some(run);
        record.status = if run_status != "completed" {
            "runtime_infrastructure_failure".into()
        } else if run_exit != case.expected_value {
            "wrong_result".into()
        } else if let Some(prefix) = &case.expected_stdout_prefix {
            if !stdout_hex.starts_with(&hex(prefix.as_bytes())) {
                "wrong_stdout".into()
            } else if case.validation_status == "validated" {
                "pass".into()
            } else {
                "exploratory_match_not_parity_pass".into()
            }
        } else if case.validation_status == "validated" {
            "pass".into()
        } else {
            "exploratory_match_not_parity_pass".into()
        };
    }
    let _ = fs::remove_dir_all(workdir);
    Ok(record)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let execute = args.iter().any(|arg| arg == "--execute");
    let allow_unvalidated = args.iter().any(|arg| arg == "--allow-unvalidated");
    let output = args
        .windows(2)
        .find(|window| window[0] == "--output")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| "parity-results.json".into());
    let timeout_seconds: u64 = args
        .windows(2)
        .find(|window| window[0] == "--timeout")
        .map(|window| window[1].parse::<u64>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(30);
    if timeout_seconds == 0 {
        return Err("--timeout must be positive".into());
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let (cases, corpus) = load_cases(&root)?;
    let timeout = Duration::from_secs(timeout_seconds);
    let mut records = Vec::new();
    for case in &cases {
        for &(backend, env_name) in &BACKENDS {
            records.push(evaluate_case(
                case,
                backend,
                env_name,
                execute,
                allow_unvalidated,
                timeout,
            )?);
        }
    }

    let mut summary = BTreeMap::new();
    for record in &records {
        *summary.entry(record.status.clone()).or_insert(0) += 1;
    }
    let report = Report {
        schema_version: 2,
        executed: execute,
        cases: cases.len(),
        corpus,
        backends: BACKENDS.iter().map(|entry| entry.0.to_string()).collect(),
        records,
        summary,
    };
    fs::write(
        &output,
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string(&report.summary).map_err(|e| e.to_string())?
    );

    let hard_fail = report.records.iter().any(|record| {
        matches!(
            record.status.as_str(),
            "compile_failure"
                | "wrong_result"
                | "wrong_stdout"
                | "unexpected_compile_success"
                | "compile_infrastructure_failure"
                | "runtime_infrastructure_failure"
        )
    });
    if hard_fail {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{hex, import_target, lowercase_kebab};

    #[test]
    fn parses_relative_import_target() {
        assert_eq!(
            import_target(r#"import fnc value from "./dep.ores";"#),
            Some("./dep.ores".into())
        );
        assert_eq!(import_target("pub fnc main(): void { return; }"), None);
    }

    #[test]
    fn enforces_lowercase_kebab_names() {
        assert!(lowercase_kebab("actor-system"));
        assert!(lowercase_kebab("nb-match"));
        assert!(!lowercase_kebab("Actor-System"));
        assert!(!lowercase_kebab("actor_system"));
        assert!(!lowercase_kebab("actor--system"));
    }

    #[test]
    fn hex_encoding_is_stable() {
        assert_eq!(hex(b"PARITY_OK"), "5041524954595f4f4b");
    }
}
