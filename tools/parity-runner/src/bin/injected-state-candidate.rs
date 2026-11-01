//! Optional execution of the seven UNVALIDATED injection candidates.
//! No test result here promotes the required manifest or proves backend parity.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const OUTPUT_LIMIT: u64 = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(30);
const BACKENDS: [(&str, &str); 5] = [
    ("java", "JAVA"), ("llvm", "LLVM"), ("wasm", "WASM"),
    ("cranelift", "CRANELIFT"), ("dotnet", "DOTNET"),
];

#[derive(Deserialize)]
struct Manifest {
    status: String,
    required_parity: bool,
    target_backends: Vec<String>,
    cases: Vec<Candidate>,
}
#[derive(Deserialize)]
struct Candidate {
    name: String,
    path: String,
    expected_kind: String,
    expected_stdout: Option<String>,
    expected_exit_code: Option<i32>,
    expected_diagnostic_category: Option<String>,
    required_phase: Option<String>,
    validation_status: String,
}
#[derive(Debug, Serialize)]
struct Receipt {
    backend: String,
    case: String,
    source_sha256: String,
    state: String,
    compiler_exit: Option<i32>,
    guest_exit: Option<i32>,
    diagnostic_category: Option<String>,
    diagnostic_phase: Option<String>,
    reason: String,
}
#[derive(Serialize)]
struct Report {
    schema_version: u32,
    required_parity: bool,
    suite_status: String,
    backends: Vec<String>,
    observed_candidates: usize,
    parity_pass: bool,
    records: Vec<Receipt>,
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self, String> {
        let n = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?.as_nanos();
        let path = env::temp_dir().join(format!("ores-injected-candidate-{}-{n}", std::process::id()));
        fs::create_dir(&path).map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}
#[derive(Default)]
struct Process {
    status: Option<i32>,
    stdout: String,
    stderr: String,
    error: Option<String>,
}
fn sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn bounded_output(path: &Path) -> Result<String, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > OUTPUT_LIMIT {
        return Err("output exceeds 1 MiB".into());
    }
    let mut text = String::new();
    File::open(path).map_err(|e| e.to_string())?
        .take(OUTPUT_LIMIT + 1).read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() as u64 > OUTPUT_LIMIT { return Err("output exceeds 1 MiB".into()); }
    Ok(text)
}
fn invoke(template: &str, source: &Path, artifact: &Path, workdir: &Path,
          logdir: &Path, phase: &str) -> Process {
    let mut result = Process::default();
    let parts = match shell_words::split(template) {
        Ok(parts) if !parts.is_empty() => parts,
        _ => { result.error = Some("empty or malformed command template".into()); return result; }
    };
    let required = if phase == "compile" { "{source}" } else { "{artifact}" };
    if !parts.iter().any(|p| p.contains(required)) {
        result.error = Some(format!("command template missing {required}"));
        return result;
    }
    let replace = |arg: &str| {
        arg.replace("{source}", &source.to_string_lossy())
           .replace("{artifact}", &artifact.to_string_lossy())
           .replace("{workdir}", &workdir.to_string_lossy())
    };
    let argv: Vec<_> = parts.iter().map(|arg| replace(arg)).collect();
    if argv.iter().any(|p| p.contains('{') || p.contains('}')) {
        result.error = Some("unknown command placeholder".into());
        return result;
    }
    let stdout = logdir.join(format!("{phase}.stdout"));
    let stderr = logdir.join(format!("{phase}.stderr"));
    let output = match File::create(&stdout) { Ok(v) => v, Err(e) => {
        result.error = Some(e.to_string()); return result;
    }};
    let errors = match File::create(&stderr) { Ok(v) => v, Err(e) => {
        result.error = Some(e.to_string()); return result;
    }};
    let mut child = match Command::new(&argv[0]).args(&argv[1..])
        .current_dir(workdir).stdout(Stdio::from(output))
        .stderr(Stdio::from(errors)).spawn() {
            Ok(v) => v,
            Err(e) => { result.error = Some(format!("spawn failed: {e}")); return result; }
        };
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(code)) => { result.status = code.code(); break; }
            Ok(None) => {
                let oversized = [&stdout, &stderr].iter().any(|p|
                    fs::metadata(p).is_ok_and(|m| m.len() > OUTPUT_LIMIT));
                if oversized || Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    result.error = Some(if oversized { "process output budget exceeded" }
                        else { "process timeout" }.into());
                    return result;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                result.error = Some(e.to_string()); return result;
            }
        }
    }
    result.stdout = match bounded_output(&stdout) {
        Ok(v) => v, Err(e) => { result.error = Some(e); return result; }
    };
    result.stderr = match bounded_output(&stderr) {
        Ok(v) => v, Err(e) => { result.error = Some(e); return result; }
    };
    if result.status.is_none() { result.error = Some("compiler/guest died by signal".into()); }
    result
}
fn diagnostic(stderr: &str) -> Option<(String, String)> {
    // A nonzero exit or parser-only diagnostic NEVER qualifies as ownership proof.
    // A reviewed compiler adapter must emit exactly one structured marker line
    // after its semantic check; ordinary console text is not sufficient.
    let mut found = None;
    for line in stderr.lines() {
        let Some(rest) = line.strip_prefix("ORES_SEMANTIC_REJECT:") else { continue };
        let mut parts = rest.split(':');
        let (Some(phase), Some(category), None) = (parts.next(), parts.next(), parts.next())
            else { return None; };
        if !matches!(phase, "type" | "ownership") ||
            !category.chars().all(|c| c.is_ascii_lowercase() || c == '_') ||
            category.is_empty() || found.is_some() {
            return None;
        }
        found = Some((phase.to_owned(), category.to_owned()));
    }
    found
}
fn safe_source(root: &Path, case: &Candidate) -> Result<PathBuf, String> {
    if !case.path.starts_with("candidates/injected-state/") ||
       !case.name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') {
        return Err("invalid candidate path/name".into());
    }
    let candidate_root = root.join("candidates/injected-state").canonicalize()
        .map_err(|e| e.to_string())?;
    let path = root.join(&case.path).canonicalize().map_err(|e| e.to_string())?;
    if !path.starts_with(&candidate_root) || path.extension().and_then(|x| x.to_str()) != Some("ores") ||
       !path.is_file() {
        return Err("candidate source path must be a regular .ores within suite".into());
    }
    Ok(path)
}
fn evaluate(case: &Candidate, compile: &Process, run: Option<&Process>,
            artifact: &Path, workdir: &Path) -> (String, String, Option<(String, String)>) {
    if let Some(e) = &compile.error {
        return ("infrastructure_failure".into(), e.clone(), None);
    }
    if case.expected_kind == "compile_reject" {
        let classified = diagnostic(&compile.stderr);
        let expected = case.expected_diagnostic_category.as_deref();
        let phase_ok = |phase: &str| match case.required_phase.as_deref() {
            Some("ownership") => phase == "ownership",
            Some("type_or_ownership") => phase == "type" || phase == "ownership",
            _ => false,
        };
        if compile.status.is_some_and(|s| s != 0)
            && compile.stdout.trim().is_empty()
            && classified.as_ref().is_some_and(|(phase, category)|
                phase_ok(phase) && expected == Some(category.as_str()))
            && !artifact.exists()
            && fs::read_dir(workdir).is_ok_and(|mut d| d.next().is_none()) {
            return ("candidate_observed_reject".into(),
                "specific semantic denial with no artifact in candidate workdir".into(),
                classified);
        }
        return ("mismatch".into(), "semantic category, phase, or output artifacts invalid".into(),
            classified);
    }
    if compile.status != Some(0) {
        return ("mismatch".into(), "positive source failed compilation".into(), None);
    }
    let Some(run) = run else {
        return ("infrastructure_failure".into(), "run command missing".into(), None);
    };
    if let Some(e) = &run.error {
        return ("infrastructure_failure".into(), e.clone(), None);
    }
    if artifact.is_file() && fs::metadata(artifact).is_ok_and(|v| v.len() > 0)
        && run.status == case.expected_exit_code
        && case.expected_stdout.as_deref() == Some(run.stdout.as_str())
        && run.stderr.is_empty() {
        ("candidate_observed_run".into(), "compiled artifact and exact guest output observed".into(), None)
    } else {
        ("mismatch".into(), "artifact or guest exit/stdout/stderr mismatch".into(), None)
    }
}
fn make_report(root: &Path, execute: bool) -> Result<Report, String> {
    let m: Manifest = serde_json::from_str(&fs::read_to_string(
        root.join("candidates/injected-state/manifest.json")).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if m.status != "draft_not_executed" || m.required_parity ||
       m.cases.len() != 7 ||
       m.target_backends != BACKENDS.iter().map(|(name, _)| name.to_string()).collect::<Vec<_>>() {
        return Err("candidate suite changed; review before executing".into());
    }
    // Pin every canonical source hash BEFORE the first backend starts.
    // A backend that rewrites an input may not establish a new reference
    // for the next backend in the matrix.
    let pinned: Vec<(PathBuf, String)> = m.cases.iter()
        .map(|case| {
            let source = safe_source(root, case)?;
            let hash = sha256(&source)?;
            Ok((source, hash))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let scratch = Scratch::new()?;
    let mut records = Vec::new();
    for (backend, env_name) in BACKENDS {
        let compile_template = env::var(format!("ORES_INJECTED_{env_name}_COMPILE")).ok();
        let run_template = env::var(format!("ORES_INJECTED_{env_name}_RUN")).ok();
        for (i, case) in m.cases.iter().enumerate() {
            if case.validation_status != "candidate_unverified" {
                return Err("unexpected validated candidate status".into());
            }
            let source = &pinned[i].0;
            let original_sha = &pinned[i].1;
            let mut receipt = Receipt {
                backend: backend.into(), case: case.name.clone(),
                source_sha256: original_sha.clone(), state: "not_run".into(),
                compiler_exit: None, guest_exit: None,
                diagnostic_category: None, diagnostic_phase: None,
                reason: "candidate not executed".into(),
            };
            if sha256(source)? != *original_sha {
                receipt.state = "source_mutated".into();
                receipt.reason = "canonical source modified by an earlier backend".into();
                records.push(receipt);
                continue;
            }
            if execute {
                if let Some(ref compile_command) = compile_template {
                    let base = scratch.0.join(format!("{backend}-{}", case.name));
                    let work = base.join("work");
                    fs::create_dir_all(&work).map_err(|e| e.to_string())?;
                    let artifact = work.join("guest-artifact");
                    let compile = invoke(compile_command, source, &artifact, &work, &base, "compile");
                    receipt.compiler_exit = compile.status;
                    let guest = if case.expected_kind == "run" && compile.status == Some(0)
                        && compile.error.is_none() {
                        run_template.as_ref().map(|command|
                            invoke(command, source, &artifact, &work, &base, "run"))
                    } else { None };
                    receipt.guest_exit = guest.as_ref().and_then(|g| g.status);
                    let (state, reason, found) = evaluate(
                        case, &compile, guest.as_ref(), &artifact, &work);
                    receipt.state = state; receipt.reason = reason;
                    if let Some((phase, category)) = found {
                        receipt.diagnostic_category = Some(category);
                        receipt.diagnostic_phase = Some(phase);
                    }
                    if sha256(source)? != *original_sha {
                        receipt.state = "source_mutated".into();
                        receipt.reason = "backend modified canonical source bytes".into();
                    }
                }
            }
            records.push(receipt);
        }
    }
    let observed = records.iter().filter(|r| r.state.starts_with("candidate_observed")).count();
    Ok(Report {
        schema_version: 1, required_parity: false, suite_status: "candidate_unverified".into(),
        backends: BACKENDS.iter().map(|(s, _)| s.to_string()).collect(),
        observed_candidates: observed, parity_pass: false, records
    })
}
fn main() {
    if let Err(e) = run() {
        eprintln!("injected-state-candidate: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let execute = args.iter().any(|a| a.to_str() == Some("--execute"));
    if args.iter().any(|a| a.to_str() != Some("--execute")) {
        return Err("usage: injected-state-candidate [--execute]".into());
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
        .canonicalize().map_err(|e| e.to_string())?;
    let report = make_report(&root, execute)?;
    let text = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    std::io::stdout().write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    println!();
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generic_parser_rejection_does_not_count() {
        assert!(diagnostic("error: unsupported syntax").is_none());
        assert!(diagnostic("ORES_SEMANTIC_REJECT:parser:use_after_move").is_none());
        assert!(diagnostic("ORES_SEMANTIC_REJECT:ownership:use_after_move").is_some());
        assert!(diagnostic("ORES_SEMANTIC_REJECT:ownership:use_after_move\nORES_SEMANTIC_REJECT:ownership:use_after_move").is_none());
    }
    #[test]
    fn source_error_and_empty_artifact_cannot_forge_positive_execution() {
        let case = Candidate { name: "positive".into(), path: "".into(),
            expected_kind: "run".into(), expected_stdout: Some("2".into()),
            expected_exit_code: Some(0), expected_diagnostic_category: None,
            required_phase: None, validation_status: "candidate_unverified".into() };
        let scratch = Scratch::new().unwrap();
        let compile = Process { status: Some(0), ..Default::default() };
        let guest = Process { status: Some(0), stdout: "2".into(), ..Default::default() };
        let artifact = scratch.0.join("absent");
        assert_eq!(evaluate(&case, &compile, Some(&guest), &artifact, &scratch.0).0, "mismatch");
    }
    #[test]
    fn expected_semantic_rejection_needs_correct_phase_and_no_artifacts() {
        let case = Candidate { name: "reject-after-take".into(), path: "".into(),
            expected_kind: "compile_reject".into(), expected_stdout: None,
            expected_exit_code: None, expected_diagnostic_category: Some("use_after_move".into()),
            required_phase: Some("ownership".into()),
            validation_status: "candidate_unverified".into() };
        let scratch = Scratch::new().unwrap();
        let workdir = scratch.0.join("work");
        fs::create_dir(&workdir).unwrap();
        let artifact = workdir.join("guest-artifact");
        let mut compiler = Process {
            status: Some(1),
            stderr: "ORES_SEMANTIC_REJECT:ownership:use_after_move\n".into(),
            ..Default::default()
        };
        assert_eq!(evaluate(&case, &compiler, None, &artifact, &workdir).0,
                   "candidate_observed_reject");
        compiler.stderr = "ORES_SEMANTIC_REJECT:type:use_after_move\n".into();
        assert_eq!(evaluate(&case, &compiler, None, &artifact, &workdir).0, "mismatch");
        compiler.stderr = "ORES_SEMANTIC_REJECT:ownership:use_after_move\n".into();
        fs::write(&artifact, b"bad ir").unwrap();
        assert_eq!(evaluate(&case, &compiler, None, &artifact, &workdir).0, "mismatch");
    }
    #[test]
    fn positive_execution_requires_artifact_and_exact_guest_output() {
        let case = Candidate { name: "positive-owned-counter".into(), path: "".into(),
            expected_kind: "run".into(), expected_stdout: Some("2".into()),
            expected_exit_code: Some(0), expected_diagnostic_category: None,
            required_phase: None, validation_status: "candidate_unverified".into() };
        let scratch = Scratch::new().unwrap();
        let artifact = scratch.0.join("guest-artifact");
        fs::write(&artifact, b"guest").unwrap();
        let compiler = Process { status: Some(0), ..Default::default() };
        let guest = Process { status: Some(0), stdout: "2".into(), ..Default::default() };
        assert_eq!(evaluate(&case, &compiler, Some(&guest), &artifact, &scratch.0).0,
                   "candidate_observed_run");
        let corrupted = Process { status: Some(0), stdout: "2\n".into(), ..Default::default() };
        assert_eq!(evaluate(&case, &compiler, Some(&corrupted), &artifact, &scratch.0).0,
                   "mismatch");
        fs::write(&artifact, b"").unwrap();
        assert_eq!(evaluate(&case, &compiler, Some(&guest), &artifact, &scratch.0).0,
                   "mismatch");
    }
    #[test]
    fn unknown_template_placeholders_fail_before_spawn() {
        let scratch = Scratch::new().unwrap();
        let source = scratch.0.join("input.ores");
        let artifact = scratch.0.join("artifact");
        fs::write(&source, b"fnc main():int{return 2;}").unwrap();
        let result = invoke(
            "untrusted-compiler {source} --output={unknown}",
            &source, &artifact, &scratch.0, &scratch.0, "compile");
        assert_eq!(result.error.as_deref(), Some("unknown command placeholder"));
    }
    #[test]
    fn no_compiler_config_is_never_a_parity_pass() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
            .canonicalize().unwrap();
        let report = make_report(&root, false).unwrap();
        assert_eq!(report.records.len(), 35);
        assert_eq!(report.observed_candidates, 0);
        assert!(!report.parity_pass);
        assert!(report.records.iter().all(|r| r.state == "not_run"));
    }
}
