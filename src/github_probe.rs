//! Read-only GitHub API metadata audit. This does not attest guest execution
//! or independently verify downloaded artifact bytes. Never elevate a report
//! claim to release evidence solely because the GitHub metadata looks valid.
use serde::Serialize;
use serde_json::Value;
use std::process::Command;

pub trait GithubReadOnly {
    fn get_json(&self, path: &str) -> Result<Value, String>;
}

/// Uses the already-authenticated GitHub CLI: only GET requests, no shell,
/// no report-supplied URLs, no mutation endpoints or arbitrary hosts.
pub struct GhCli;

impl GithubReadOnly for GhCli {
    fn get_json(&self, path: &str) -> Result<Value, String> {
        if !path.starts_with("/repos/") || path.contains("..")
            || path.contains('%') || path.contains('?') && !path.ends_with("?per_page=100")
        {
            return Err("unapproved GitHub API path".into());
        }
        let out = Command::new("gh")
            .arg("api")
            .arg("-X").arg("GET")
            .arg("-H").arg("Accept: application/vnd.github+json")
            .arg(path)
            .env("GH_PROMPT_DISABLED", "1")
            .output().map_err(|e| format!("cannot execute gh api: {e}"))?;
        if !out.status.success() {
            return Err(format!("read-only GitHub API rejected path ({})", out.status));
        }
        if out.stdout.len() > 2_097_152 {
            return Err("GitHub response exceeds two MiB".into());
        }
        serde_json::from_slice(&out.stdout).map_err(|e| format!("invalid GitHub response: {e}"))
    }
}

#[derive(Debug, Clone)]
pub struct ProbeSpec<'a> {
    pub repository: &'a str,
    pub pr_number: u64,
    pub run_id: u64,
    pub source_sha: &'a str,
    pub source_tree_sha: &'a str,
    pub workflow_path: &'a str,
    pub job_name: &'a str,
    pub required_steps: &'a [&'a str],
    pub approved_reviewer: &'a str,
    pub artifact_name: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeResult {
    pub repository: String,
    pub source_sha: String,
    pub tree_sha: String,
    pub run_id: u64,
    pub run_attempt: u64,
    pub job_id: u64,
    pub artifact_id: u64,
    pub artifact_digest_metadata: String,
    pub independent_review_found: bool,
    pub branch_protection_verified: bool,
    pub required_steps_verified: bool,
    pub artifact_bytes_verified: bool,
    pub executed_guest_verified: bool,
    pub release_certification: String,
}

fn require(value: bool, reason: &str) -> Result<(), String> {
    if value { Ok(()) } else { Err(reason.into()) }
}
fn str_field<'a>(v: &'a Value, path: &[&str]) -> &'a str {
    let mut x = v;
    for p in path { x = &x[*p]; }
    x.as_str().unwrap_or("")
}
fn num_field(v: &Value, path: &[&str]) -> u64 {
    let mut x = v;
    for p in path { x = &x[*p]; }
    x.as_u64().unwrap_or(0)
}
fn valid_sha(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn valid_repo(s: &str) -> bool {
    let mut parts = s.split('/');
    let (Some(a), Some(b), None)=(parts.next(),parts.next(),parts.next()) else {return false};
    !a.is_empty() && !b.is_empty() && a.len()<=39 && b.len()<=100
        && a.bytes().all(|x| x.is_ascii_alphanumeric() || x==b'-')
        && b.bytes().all(|x| x.is_ascii_alphanumeric() || x==b'-' || x==b'_' || x==b'.')
        && !a.starts_with('-') && !a.ends_with('-')
        && b!="." && b!=".."
}
fn safe_name(s: &str) -> bool {
    !s.is_empty() && s.len()<=120 && s.bytes().all(|b| b.is_ascii_alphanumeric()
        || matches!(b, b'_'|b'-'|b'.'|b'/'))
}
fn fetched<A: GithubReadOnly>(api: &A, repo: &str, path: &str) -> Result<Value,String> {
    api.get_json(&format!("/repos/{repo}/{path}"))
}

pub fn probe_github_metadata<A: GithubReadOnly>(
    api: &A, s: &ProbeSpec<'_>,
) -> Result<ProbeResult, String> {
    require(valid_repo(s.repository), "invalid trusted GitHub repository")?;
    require(s.pr_number>0 && s.run_id>0, "missing PR or run ID")?;
    require(valid_sha(s.source_sha,40) && valid_sha(s.source_tree_sha,40),
        "invalid expected source commit/tree SHA")?;
    require(s.workflow_path.starts_with(".github/workflows/")
        && safe_name(s.workflow_path) && safe_name(s.job_name)
        && safe_name(s.artifact_name) && safe_name(s.approved_reviewer)
        && !s.required_steps.is_empty()
        && s.required_steps.iter().all(|v| !v.is_empty() && v.len()<=256),
        "invalid workflow, review, step or artifact selector")?;

    let pr = fetched(api,s.repository,&format!("pulls/{}",s.pr_number))?;
    require(str_field(&pr,&["head","sha"])==s.source_sha,"PR head is stale or unexpected")?;
    require(str_field(&pr,&["base","ref"])=="main","unexpected target branch")?;
    require(str_field(&pr,&["state"])=="open" && pr["draft"]==false,
        "PR is closed or not ready for review")?;
    let author=str_field(&pr,&["user","login"]);
    require(!author.is_empty() && !author.eq_ignore_ascii_case(s.approved_reviewer),
        "cannot use PR author's own approval")?;

    let commit=fetched(api,s.repository,&format!("git/commits/{}",s.source_sha))?;
    require(str_field(&commit,&["sha"])==s.source_sha
        && str_field(&commit,&["tree","sha"])==s.source_tree_sha,
        "source commit/tree does not match exact expected tree")?;

    let run=fetched(api,s.repository,&format!("actions/runs/{}",s.run_id))?;
    let attempt=num_field(&run,&["run_attempt"]);
    require(num_field(&run,&["id"])==s.run_id && attempt>0
        && str_field(&run,&["repository","full_name"]).eq_ignore_ascii_case(s.repository)
        && str_field(&run,&["head_sha"])==s.source_sha
        && str_field(&run,&["status"])=="completed"
        && str_field(&run,&["conclusion"])=="success"
        && str_field(&run,&["path"])==s.workflow_path
        && str_field(&run,&["event"])=="pull_request",
        "workflow run not successful, exact-head, or trusted")?;

    let jobs=fetched(api,s.repository,
        &format!("actions/runs/{}/jobs?per_page=100",s.run_id))?;
    let list=jobs["jobs"].as_array().ok_or("malformed jobs list")?;
    require(!list.is_empty() && list.len()<100
        && num_field(&jobs,&["total_count"])==list.len() as u64,
        "missing or truncated GitHub jobs page")?;
    let matches:Vec<_>=list.iter().filter(|j|str_field(j,&["name"])==s.job_name).collect();
    require(matches.len()==1,"expected exactly one designated evidence job")?;
    let job=matches[0];
    require(num_field(job,&["id"])>0 && num_field(job,&["run_id"])==s.run_id
        && num_field(job,&["run_attempt"])==attempt
        && str_field(job,&["head_sha"])==s.source_sha
        && str_field(job,&["status"])=="completed"
        && str_field(job,&["conclusion"])=="success",
        "evidence job is incomplete, stale or failed")?;
    let steps=job["steps"].as_array().ok_or("missing executed job steps")?;
    for name in s.required_steps {
        let found=steps.iter().filter(|step|str_field(step,&["name"])==*name
            && str_field(step,&["status"])=="completed"
            && str_field(step,&["conclusion"])=="success").count();
        require(found==1,"required executed workflow step missing or ambiguous")?;
    }

    let reviews=fetched(api,s.repository,
        &format!("pulls/{}/reviews?per_page=100",s.pr_number))?;
    let reviews=reviews.as_array().ok_or("invalid reviews listing")?;
    require(reviews.len()<100,"review page truncated; pagination required")?;
    require(reviews.iter().any(|v| {
        str_field(v,&["user","login"]).eq_ignore_ascii_case(s.approved_reviewer)
            && str_field(v,&["state"])=="APPROVED"
            && str_field(v,&["commit_id"])==s.source_sha
            && num_field(v,&["id"])>0
    }),"independent approval on exact PR head is absent")?;

    // Github's protected-branch endpoint requires admin permissions.
    // Any 403, 404, incomplete response or weak policy MUST reject.
    let protection=fetched(api,s.repository,"branches/main/protection")?;
    require(num_field(&protection,
        &["required_pull_request_reviews","required_approving_review_count"])>=1
        && protection["enforce_admins"]["enabled"]==true
        && protection["required_status_checks"]["strict"]==true,
        "protected main with independent review and strict CI is not established")?;

    let artifacts=fetched(api,s.repository,
        &format!("actions/runs/{}/artifacts?per_page=100",s.run_id))?;
    let list=artifacts["artifacts"].as_array().ok_or("malformed artifacts list")?;
    require(list.len()<100 && num_field(&artifacts,&["total_count"])==list.len() as u64,
        "missing or truncated artifacts")?;
    let matches:Vec<_>=list.iter().filter(|a|str_field(a,&["name"])==s.artifact_name
        && a["expired"]==false
        && num_field(a,&["workflow_run","id"])==s.run_id
        && str_field(a,&["digest"]).starts_with("sha256:")
        && valid_sha(str_field(a,&["digest"]).trim_start_matches("sha256:"),64)
        && num_field(a,&["id"])>0).collect();
    require(matches.len()==1,"required artifact digest metadata absent or ambiguous")?;
    let artifact=matches[0];

    Ok(ProbeResult {
        repository:s.repository.into(),source_sha:s.source_sha.into(),
        tree_sha:s.source_tree_sha.into(),run_id:s.run_id,run_attempt:attempt,
        job_id:num_field(job,&["id"]),artifact_id:num_field(artifact,&["id"]),
        artifact_digest_metadata:str_field(artifact,&["digest"]).into(),
        independent_review_found:true,branch_protection_verified:true,
        required_steps_verified:true,
        artifact_bytes_verified:false,executed_guest_verified:false,
        release_certification:"NOT_CERTIFIED".into(),
    })
}
