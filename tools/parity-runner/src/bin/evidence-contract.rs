use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process;

const SCHEMA: &str = "oreslang-parity-execution-evidence-v1";
const TIERS: [&str; 4] = [
    "RUNNER_EXECUTED",
    "HOST_TESTS_EXECUTED",
    "NATIVE_COMPILER_BUILT",
    "ORES_GUEST_EXECUTED",
];
const CATEGORIES: [&str; 5] = [
    "setup_contract_policy_source_free_canary",
    "host_compiler_tests",
    "native_compiler_build",
    "compiled_ores_guest_runtime_execution",
    "indeterminate",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    schema: String,
    claim_tier: String,
    backend: String,
    repository: String,
    repository_id: u64,
    run_id: u64,
    run_attempt: u64,
    workflow_id: u64,
    workflow_path: String,
    workflow_run_path: String,
    event: String,
    head_commit: String,
    head_tree: String,
    workflow_blob_sha: String,
    verified_jobs: usize,
    successful_non_housekeeping_steps: usize,
    jobs: Vec<JobEvidence>,
    artifacts_verified: bool,
    #[serde(default)]
    artifacts: Vec<ArtifactEvidence>,
    #[serde(default)]
    compiler: Option<CompilerEvidence>,
    #[serde(default)]
    guest: Option<GuestEvidence>,
    statement: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JobEvidence {
    job_id: u64,
    job_name: String,
    runner_id: u64,
    runner_name: String,
    conclusion: String,
    started_at: String,
    completed_at: String,
    meaningful_steps: Vec<StepEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StepEvidence {
    name: String,
    category: String,
    conclusion: String,
    started_at: String,
    completed_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactEvidence {
    id: u64,
    name: String,
    expired: bool,
    #[serde(default)]
    sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompilerEvidence {
    artifact_id: u64,
    identity: String,
    sha256: String,
    source_head_sha: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GuestEvidence {
    case_id: String,
    source_sha256: String,
    artifact_id: u64,
    artifact_sha256: String,
    exit_code: i32,
    expected_exit_code: i32,
    stdout_hex: String,
    expected_stdout_hex: String,
}

fn backend_identity(name: &str) -> Option<(&'static str, u64, &'static str)> {
    match name {
        "java" => Some((
            "ores-truffle-oreslang/oreslang-source.java",
            1400792274,
            "java",
        )),
        "llvm" => Some((
            "ores-truffle-oreslang/oreslang-source.llvm",
            1410897874,
            "llvm",
        )),
        "net" => Some((
            "ores-truffle-oreslang/oreslang-source.net",
            1419704375,
            "dotnet",
        )),
        "wasm" => Some((
            "ores-truffle-oreslang/oreslang-source.wasm",
            1419858954,
            "wasm",
        )),
        "forge" => Some((
            "ores-truffle-oreslang/oreslang-source.forge",
            1419703674,
            "forge",
        )),
        "cranelift" => Some((
            "ores-truffle-oreslang/oreslang-source.cranelift",
            1437273004,
            "cranelift",
        )),
        _ => None,
    }
}

fn nonblank(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(|ch| ch.is_control())
}

fn lower_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn utc_timestamp(value: &str) -> bool {
    // Require real Gregorian UTC dates, not only a fixed timestamp shape.
    if value.len() != 20 {
        return false;
    }
    let bytes = value.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        match index {
            4 | 7 if *byte != b'-' => return false,
            10 if *byte != b'T' => return false,
            13 | 16 if *byte != b':' => return false,
            19 if *byte != b'Z' => return false,
            4 | 7 | 10 | 13 | 16 | 19 => {}
            _ if !byte.is_ascii_digit() => return false,
            _ => {}
        }
    }
    let year = value[0..4].parse::<u32>().unwrap_or(0);
    let month = value[5..7].parse::<u32>().unwrap_or(0);
    let day = value[8..10].parse::<u32>().unwrap_or(0);
    let hour = value[11..13].parse::<u32>().unwrap_or(24);
    let minute = value[14..16].parse::<u32>().unwrap_or(60);
    let second = value[17..19].parse::<u32>().unwrap_or(60);
    if year == 0 || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    day >= 1 && day <= days
}

fn safe_workflow_path(path: &str) -> bool {
    if !path.starts_with(".github/workflows/")
        || !(path.ends_with(".yml") || path.ends_with(".yaml"))
        || path.contains("//")
    {
        return false;
    }
    !path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
}

fn safe_run_path(actual: &str, workflow: &str) -> bool {
    if actual == workflow {
        return true;
    }
    let Some(rest) = actual.strip_prefix(&(workflow.to_string() + "@")) else {
        return false;
    };
    !rest.is_empty()
        && !rest
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        && rest
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-./".contains(&b))
}

fn housekeeping(name: &str) -> bool {
    let folded = name.trim().to_ascii_lowercase();
    folded == "set up job"
        || folded == "complete job"
        || folded.starts_with("post ")
        || folded.starts_with("pre ")
        || folded == "initialize containers"
        || folded == "start containers"
        || folded == "stop containers"
}

fn tier_rank(tier: &str) -> Option<u8> {
    TIERS
        .iter()
        .position(|candidate| *candidate == tier)
        .map(|n| n as u8)
}

fn artifact<'a>(evidence: &'a Evidence, id: u64) -> Option<&'a ArtifactEvidence> {
    evidence.artifacts.iter().find(|item| item.id == id)
}

fn validate(e: &Evidence) -> Result<&'static str, String> {
    if e.schema != SCHEMA {
        return Err("unsupported evidence schema".into());
    }
    let rank = tier_rank(&e.claim_tier).ok_or("unsupported claim tier")?;
    let (expected_repo, expected_repo_id, parity_backend) =
        backend_identity(&e.backend).ok_or("unsupported source backend")?;
    if e.repository != expected_repo || e.repository_id != expected_repo_id {
        return Err("backend repository identity mismatch".into());
    }
    for (label, value) in [
        ("run_id", e.run_id),
        ("run_attempt", e.run_attempt),
        ("workflow_id", e.workflow_id),
    ] {
        if value == 0 {
            return Err(format!("{label} must be positive"));
        }
    }
    if !lower_hex(&e.head_commit, 40)
        || !lower_hex(&e.head_tree, 40)
        || !lower_hex(&e.workflow_blob_sha, 40)
    {
        return Err("commit/tree/workflow blob identity is malformed".into());
    }
    if e.event != "push" && e.event != "pull_request" {
        return Err("unsupported workflow event".into());
    }
    if !safe_workflow_path(&e.workflow_path)
        || !safe_run_path(&e.workflow_run_path, &e.workflow_path)
    {
        return Err("workflow path identity is malformed".into());
    }
    if e.statement != "GITHUB_RUN_EXECUTION_VERIFIED_NOT_GUEST_PARITY_CERTIFIED" {
        return Err("unexpected source verifier statement".into());
    }
    if e.jobs.is_empty() || e.verified_jobs != e.jobs.len() {
        return Err("verified job count mismatch".into());
    }

    let mut job_ids = BTreeSet::new();
    let mut job_names = BTreeSet::new();
    let mut categories = BTreeSet::new();
    let mut meaningful_count = 0usize;
    for job in &e.jobs {
        if job.job_id == 0 || job.runner_id == 0 {
            return Err("job and runner IDs must be positive".into());
        }
        if !job_ids.insert(job.job_id) || !job_names.insert(job.job_name.clone()) {
            return Err("duplicate job identity".into());
        }
        if !nonblank(&job.job_name) || !nonblank(&job.runner_name) {
            return Err("job or runner name is blank/malformed".into());
        }
        if job.conclusion != "success" {
            return Err("verified job did not conclude successfully".into());
        }
        if !utc_timestamp(&job.started_at)
            || !utc_timestamp(&job.completed_at)
            || job.completed_at < job.started_at
        {
            return Err("job execution timestamps are malformed".into());
        }
        if job.meaningful_steps.is_empty() {
            return Err("verified job has no meaningful executed step".into());
        }
        let mut step_names = BTreeSet::new();
        for step in &job.meaningful_steps {
            meaningful_count += 1;
            if !nonblank(&step.name) || housekeeping(&step.name) {
                return Err("housekeeping or malformed step cannot prove execution".into());
            }
            if !step_names.insert(step.name.clone()) {
                return Err("duplicate meaningful step name in job".into());
            }
            if !CATEGORIES.contains(&step.category.as_str()) {
                return Err("unsupported meaningful-step category".into());
            }
            categories.insert(step.category.as_str());
            if step.conclusion != "success" {
                return Err("meaningful proof step did not conclude successfully".into());
            }
            if !utc_timestamp(&step.started_at)
                || !utc_timestamp(&step.completed_at)
                || step.completed_at < step.started_at
                || step.started_at < job.started_at
                || step.completed_at > job.completed_at
            {
                return Err("meaningful step timestamps are malformed".into());
            }
        }
    }
    if meaningful_count != e.successful_non_housekeeping_steps {
        return Err("meaningful step count mismatch".into());
    }

    let mut artifact_ids = BTreeSet::new();
    for item in &e.artifacts {
        if item.id == 0 || !artifact_ids.insert(item.id) || !nonblank(&item.name) {
            return Err("artifact identity is malformed or duplicated".into());
        }
        if let Some(digest) = &item.sha256 {
            if !lower_hex(digest, 64) {
                return Err("artifact sha256 is malformed".into());
            }
        }
    }

    if rank >= 1 && !categories.contains("host_compiler_tests") {
        return Err("HOST_TESTS_EXECUTED requires a host compiler test step".into());
    }

    if rank >= 2 {
        if !e.artifacts_verified {
            return Err("compiler/guest tiers require verified artifact enumeration".into());
        }
        if !categories.contains("native_compiler_build") {
            return Err("NATIVE_COMPILER_BUILT requires a native compiler build step".into());
        }
        let compiler = e
            .compiler
            .as_ref()
            .ok_or("compiler provenance is required")?;
        if compiler.artifact_id == 0
            || !nonblank(&compiler.identity)
            || !lower_hex(&compiler.sha256, 64)
            || compiler.source_head_sha != e.head_commit
        {
            return Err("compiler provenance is malformed".into());
        }
        let item = artifact(e, compiler.artifact_id).ok_or("compiler artifact is missing")?;
        if item.expired || item.sha256.as_deref() != Some(compiler.sha256.as_str()) {
            return Err("compiler artifact provenance mismatch".into());
        }
    }

    if rank >= 3 {
        if !categories.contains("compiled_ores_guest_runtime_execution") {
            return Err("ORES_GUEST_EXECUTED requires a compiled guest execution step".into());
        }
        let guest = e
            .guest
            .as_ref()
            .ok_or("compiled guest provenance is required")?;
        if !nonblank(&guest.case_id)
            || !lower_hex(&guest.source_sha256, 64)
            || !lower_hex(&guest.artifact_sha256, 64)
            || !lower_hex(&guest.stdout_hex, guest.stdout_hex.len())
            || !lower_hex(&guest.expected_stdout_hex, guest.expected_stdout_hex.len())
            || guest.stdout_hex.len() % 2 != 0
            || guest.expected_stdout_hex.len() % 2 != 0
        {
            return Err("compiled guest provenance is malformed".into());
        }
        if guest.exit_code != guest.expected_exit_code
            || guest.stdout_hex != guest.expected_stdout_hex
        {
            return Err("compiled guest oracle mismatch".into());
        }
        let item = artifact(e, guest.artifact_id).ok_or("compiled guest artifact is missing")?;
        if item.expired || item.sha256.as_deref() != Some(guest.artifact_sha256.as_str()) {
            return Err("compiled guest artifact provenance mismatch".into());
        }
    }

    Ok(parity_backend)
}

fn validate_report(evidence: &Evidence, parity_backend: &str, path: &str) -> Result<(), String> {
    let raw = fs::read(path).map_err(|e| format!("failed to read parity report: {e}"))?;
    if raw.is_empty() || raw.len() > 16 * 1024 * 1024 {
        return Err("parity report is empty or oversized".into());
    }
    let report: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|e| format!("invalid parity report JSON: {e}"))?;
    if report.get("schema_version").and_then(|value| value.as_u64()) != Some(2) {
        return Err("compiled guest evidence requires parity report schema v2".into());
    }
    if report.get("executed") != Some(&serde_json::Value::Bool(true)) {
        return Err("compiled guest evidence requires an executed parity report".into());
    }
    let backends = report
        .get("backends")
        .and_then(|value| value.as_array())
        .ok_or("parity report has no backend list")?;
    if !backends
        .iter()
        .any(|value| value.as_str() == Some(parity_backend))
    {
        return Err("parity report does not include evidence backend".into());
    }
    let guest = evidence
        .guest
        .as_ref()
        .ok_or("compiled guest provenance is required")?;
    let records = report
        .get("records")
        .and_then(|value| value.as_array())
        .ok_or("parity report has no records")?;
    let matches: Vec<_> = records
        .iter()
        .filter(|record| {
            record.get("backend").and_then(|v| v.as_str()) == Some(parity_backend)
                && record.get("case_id").and_then(|v| v.as_str()) == Some(guest.case_id.as_str())
                && record.get("source_sha256").and_then(|v| v.as_str())
                    == Some(guest.source_sha256.as_str())
        })
        .collect();
    if matches.len() != 1 {
        return Err("compiled guest evidence must match exactly one parity-report cell".into());
    }
    let record = matches[0];
    if record.get("status").and_then(|v| v.as_str()) != Some("pass")
        || record.get("validation_status").and_then(|v| v.as_str()) != Some("validated")
    {
        return Err("compiled guest parity-report cell is not a validated pass".into());
    }
    let compile = record
        .get("compile")
        .and_then(|value| value.as_object())
        .ok_or("compiled guest parity-report cell has no compiler invocation")?;
    if compile.get("status").and_then(|v| v.as_str()) != Some("completed")
        || compile.get("exit_code").and_then(|v| v.as_i64()) != Some(0)
    {
        return Err("compiled guest parity-report compiler did not succeed".into());
    }
    let invocation = record
        .get("run")
        .and_then(|value| value.as_object())
        .ok_or("compiled guest parity-report cell has no runtime invocation")?;
    if invocation.get("status").and_then(|v| v.as_str()) != Some("completed")
        || invocation.get("exit_code").and_then(|v| v.as_i64()) != Some(guest.exit_code as i64)
        || invocation.get("stdout_hex").and_then(|v| v.as_str()) != Some(guest.stdout_hex.as_str())
    {
        return Err("compiled guest runtime evidence disagrees with parity report".into());
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: evidence-contract <evidence.json> [--report <parity-results.json>]")?;
    let mut report_path = None;
    while let Some(arg) = args.next() {
        if arg != "--report" || report_path.is_some() {
            return Err(
                "usage: evidence-contract <evidence.json> [--report <parity-results.json>]".into(),
            );
        }
        report_path = Some(
            args.next()
                .ok_or("--report requires a parity-results.json path")?,
        );
    }
    let raw = fs::read(&path).map_err(|e| format!("failed to read evidence: {e}"))?;
    if raw.is_empty() || raw.len() > 1024 * 1024 {
        return Err("evidence file is empty or oversized".into());
    }
    let evidence: Evidence =
        serde_json::from_slice(&raw).map_err(|e| format!("invalid evidence JSON: {e}"))?;
    let parity_backend = validate(&evidence)?;
    if evidence.claim_tier == "ORES_GUEST_EXECUTED" {
        let path = report_path
            .as_deref()
            .ok_or("ORES_GUEST_EXECUTED requires --report binding")?;
        validate_report(&evidence, parity_backend, path)?;
    }
    println!(
        "{}",
        json!({
            "valid": true,
            "schema": SCHEMA,
            "source_backend": evidence.backend,
            "parity_backend": parity_backend,
            "claim_tier": evidence.claim_tier,
            "run_id": evidence.run_id,
            "run_attempt": evidence.run_attempt,
        })
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("PARITY_EXECUTION_EVIDENCE_REJECTED: {error}");
        process::exit(1);
    }
}
