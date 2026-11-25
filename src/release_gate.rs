//! Offline, fail-closed *consistency* gate for trusted, separately retrieved
//! GitHub execution evidence. No HTTP client or external attestation is supplied.
use serde_json::Value;
use crate::decode_and_validate;

#[derive(Clone, Debug)]
pub struct ReleasePolicy<'a> {
    pub trusted_repository: &'a str,
    pub source_sha: &'a str,
    pub case_set_sha256: &'a str,
    pub backend: &'a str,
    pub feature_id: &'a str,
    pub require_guest: bool,
}

#[derive(Clone, Debug)]
pub struct ObservedRun<'a> {
    /// All of these fields MUST be obtained from a trusted read-only GitHub
    /// client and independent artifact inspection, NOT from the report.
    pub repository: &'a str,
    pub run_id: u64,
    pub head_sha: &'a str,
    pub complete: bool,
    pub workflow_conclusion: &'a str,
    pub test_job_conclusion: &'a str,
    pub test_steps_executed: bool,
    pub execution_target: &'a str,
    pub executed_cases: u64,
    pub failed_cases: u64,
    pub case_set_sha256: &'a str,
    pub artifact_digests_verified: bool,
    pub independent_reviews_verified: bool,
    pub protected_main_verified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedEvidence {
    pub backend: String,
    pub feature_id: String,
    pub source_sha: String,
    pub github_run: String,
    pub executed_cases: u64,
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str)
        .ok_or_else(|| format!("missing or invalid {key}"))
}

fn valid_repository(name: &str) -> bool {
    let mut parts = name.split('/');
    let Some(owner) = parts.next() else { return false };
    let Some(repo) = parts.next() else { return false };
    if parts.next().is_some() { return false; }
    if owner.is_empty() || repo.is_empty() || owner.len() > 39 || repo.len() > 100 {
        return false;
    }
    owner.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        && repo.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'.')
        && !owner.starts_with('-') && !owner.ends_with('-')
        && repo != "." && repo != ".."
}

/// This is not a network attestor: it refuses a false/unsupported claim even
/// when well-shaped JSON is supplied, but the caller must supply independently
/// retrieved and verified GitHub metadata. Never fetch an untrusted report URL.
pub fn accept_verified_passing_evidence(
    report_bytes: &[u8],
    schema: &Value,
    policy: &ReleasePolicy<'_>,
    observed: &ObservedRun<'_>,
) -> Result<AcceptedEvidence, String> {
    let report = decode_and_validate(report_bytes, schema)?;
    if field(&report, "status")? != "passed" {
        return Err("only a completed, verified passing guest run may be accepted".into());
    }
    if !valid_repository(policy.trusted_repository)
        || observed.repository != policy.trusted_repository
        || observed.run_id == 0
    {
        return Err("untrusted repository or missing run identifier".into());
    }
    if field(&report, "backend")? != policy.backend
        || field(&report, "feature_id")? != policy.feature_id
        || field(&report, "source_sha")? != policy.source_sha
        || field(&report, "case_set_sha256")? != policy.case_set_sha256
        || observed.head_sha != policy.source_sha
        || observed.case_set_sha256 != policy.case_set_sha256
    {
        return Err("stale or mismatched source, feature, backend, or fixture".into());
    }
    let run_url = format!("https://github.com/{}/actions/runs/{}",
        policy.trusted_repository, observed.run_id);
    if field(&report, "run_url")? != run_url {
        return Err("report URL does not match trusted GitHub run".into());
    }
    if !observed.complete || observed.workflow_conclusion != "success"
        || observed.test_job_conclusion != "success" || !observed.test_steps_executed
    {
        return Err("GitHub workflow or required steps not confirmed successful".into());
    }
    let target = field(&report, "execution_target")?;
    if target != observed.execution_target
        || (policy.require_guest && target != "guest")
    {
        return Err("guest/host execution target mismatch".into());
    }
    let report_executed = report["executed_cases"].as_u64()
        .ok_or("missing executed case count")?;
    let report_failed = report["failed_cases"].as_u64()
        .ok_or("missing failed case count")?;
    if report_executed == 0 || report_failed != 0
        || report_executed != observed.executed_cases
        || observed.failed_cases != 0
    {
        return Err("executed or failed case count does not match verified run".into());
    }
    if !observed.artifact_digests_verified || !observed.independent_reviews_verified
        || !observed.protected_main_verified
    {
        return Err("artifact provenance, independent review, or branch protection absent".into());
    }
    Ok(AcceptedEvidence {
        backend: policy.backend.into(),
        feature_id: policy.feature_id.into(),
        source_sha: policy.source_sha.into(),
        github_run: run_url,
        executed_cases: report_executed,
    })
}
