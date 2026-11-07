//! Read-only GitHub evidence metadata inspector. No release certification is issued.
//! Fetch only fixed api.github.com endpoints; NEVER open URLs from submitted reports.
use crate::decode_strict_json;
use serde::Deserialize;
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    pub repository: String,
    pub pull_number: u64,
    pub run_id: u64,
    pub source_sha: String,
    pub tree_sha: String,
    pub job_name: String,
    pub step_name: String,
    pub artifact_name: String,
    pub reviewer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataReceipt {
    pub repository: String,
    pub run_id: u64,
    pub head_sha: String,
    pub tree_sha: String,
    pub job_id: u64,
    pub runner_name: String,
    pub artifact_id: u64,
    pub artifact_digest_claim: String,
    // Never set these to true based only on GitHub REST metadata.
    pub artifact_bytes_verified: bool,
    pub guest_execution_verified: bool,
}
pub trait GitHubReader { fn get(&self, path: &str) -> Result<Value, String>; }

pub struct CurlGitHub { token: String }
impl CurlGitHub {
    pub fn from_environment() -> Result<Self, String> {
        let token = std::env::var("GH_TOKEN")
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .map_err(|_| "missing read-only GitHub API token")?;
        Self::new(token)
    }
    pub fn new(token: String) -> Result<Self, String> {
        if token.is_empty() || token.len() > 1024 || !token.bytes().all(
            |c| c.is_ascii_graphic() && c != b'"' && c != b'\\') {
            return Err("malformed GitHub API token".into());
        }
        Ok(Self { token })
    }
}
impl GitHubReader for CurlGitHub {
    fn get(&self, path: &str) -> Result<Value, String> {
        if !path.starts_with("/repos/") || path.contains("//") || path.contains("..")
            || !path.bytes().all(|c| c.is_ascii_alphanumeric()
                || matches!(c, b'/'|b'-'|b'_'|b'.'|b'?'|b'='|b'&')) {
            return Err("invalid GitHub API path".into());
        }
        let mut proc = Command::new("curl")
            .arg("-q") // ignore ~/.curlrc; first argument is required
            .args(["--silent","--show-error","--fail-with-body",
                "--max-time","20","--max-filesize","1048576","--proto","=https",
                "--request","GET","--header","Accept: application/vnd.github+json",
                "--header","X-GitHub-Api-Version: 2022-11-28",
                "--header","User-Agent: atto-evidence-inspect/0.1",
                "--config","-"])
            .arg(format!("https://api.github.com{path}"))
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
            .spawn().map_err(|_| "cannot start read-only GitHub transport")?;
        let config = format!("header = \"Authorization: Bearer {}\"\n", self.token);
        proc.stdin.take().ok_or("missing transport stdin")?
            .write_all(config.as_bytes()).map_err(|_| "cannot configure transport")?;
        let out = proc.wait_with_output().map_err(|_| "GitHub API transport failed")?;
        if !out.status.success() { return Err("GitHub GET failed or response truncated".into()); }
        decode_strict_json(&out.stdout)
    }
}
fn get_s<'a>(v: &'a Value, keys: &[&str]) -> Result<&'a str, String> {
    let mut x = v;
    for k in keys { x=x.get(*k).ok_or_else(|| format!("missing {}",keys.join(".")))?; }
    x.as_str().ok_or_else(|| format!("invalid {}",keys.join(".")))
}
fn get_n(v: &Value, keys: &[&str]) -> Result<u64, String> {
    let mut x = v;
    for k in keys { x=x.get(*k).ok_or_else(|| format!("missing {}",keys.join(".")))?; }
    x.as_u64().ok_or_else(|| format!("invalid {}",keys.join(".")))
}
fn eq(v: &Value, keys: &[&str], expected: &str) -> Result<(), String> {
    if get_s(v,keys)? != expected { return Err(format!("GitHub {} mismatch",keys.join("."))); }
    Ok(())
}
fn arr<'a>(v: &'a Value, key: &str) -> Result<&'a [Value], String> {
    v.get(key).and_then(Value::as_array).map(Vec::as_slice)
        .ok_or_else(|| format!("missing {key} list"))
}
fn good_repo(s: &str) -> bool {
    let p: Vec<_> = s.split('/').collect();
    if p.len()!=2 || p[0].is_empty() || p[1].is_empty() ||
        p[0].len()>39 || p[1].len()>100 || p[1]=="." || p[1]==".." { return false; }
    p[0].bytes().all(|b| b.is_ascii_alphanumeric()||b==b'-')
      && p[1].bytes().all(|b| b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_'|b'.'))
      && !p[0].starts_with('-') && !p[0].ends_with('-')
}
fn hex40(s: &str) -> bool { s.len()==40 && s.bytes().all(|b|b.is_ascii_hexdigit()) }
fn safe(s: &str) -> bool { !s.is_empty()&&s.len()<=128&&!s.chars().any(char::is_control) }

/// Independently read GitHub REST *metadata*; do not claim guest execution or verified artifact bytes.
pub fn inspect(client: &impl GitHubReader, p: &Expected) -> Result<MetadataReceipt, String> {
    if !good_repo(&p.repository)||p.pull_number==0||p.run_id==0 ||
       !hex40(&p.source_sha)||!hex40(&p.tree_sha)||
       !safe(&p.job_name)||!safe(&p.step_name)||!safe(&p.artifact_name)||!safe(&p.reviewer) {
        return Err("malformed trusted verifier policy".into());
    }
    let base=format!("/repos/{}",p.repository);
    let run_path=format!("{base}/actions/runs/{}",p.run_id);
    let run=client.get(&run_path)?;
    if get_n(&run,&["id"])?!=p.run_id {return Err("run ID mismatch".into());}
    let attempt=get_n(&run,&["run_attempt"])?;
    if attempt==0 {return Err("unknown GitHub run attempt".into());}
    eq(&run,&["repository","full_name"],&p.repository)?;
    eq(&run,&["head_sha"],&p.source_sha)?;
    eq(&run,&["status"],"completed")?;
    eq(&run,&["conclusion"],"success")?;

    let commit=client.get(&format!("{base}/git/commits/{}",p.source_sha))?;
    eq(&commit,&["sha"],&p.source_sha)?;
    eq(&commit,&["tree","sha"],&p.tree_sha)?;

    let pull=client.get(&format!("{base}/pulls/{}",p.pull_number))?;
    eq(&pull,&["head","sha"],&p.source_sha)?;
    eq(&pull,&["state"],"open")?;
    eq(&pull,&["base","ref"],"main")?;
    if pull.get("draft").and_then(Value::as_bool)!=Some(false) {
        return Err("draft/unknown PR state".into());
    }
    if get_s(&pull,&["user","login"])?.eq_ignore_ascii_case(&p.reviewer) {
        return Err("PR author cannot independently approve".into());
    }

    let jobs=client.get(&format!("{run_path}/jobs?per_page=100&page=1&filter=latest"))?;
    let list=arr(&jobs,"jobs")?;
    if get_n(&jobs,&["total_count"])?!=list.len() as u64 {
        return Err("job pagination incomplete".into());
    }
    let matches:Vec<_>=list.iter().filter(|j|j["name"].as_str()==Some(&p.job_name)).collect();
    if matches.len()!=1 {return Err("required job missing/ambiguous".into());}
    let job=matches[0];
    if get_n(job,&["run_id"])?!=p.run_id||get_n(job,&["run_attempt"])?!=attempt {
        return Err("job from wrong run attempt".into());
    }
    eq(job,&["status"],"completed")?; eq(job,&["conclusion"],"success")?;
    let job_id=get_n(job,&["id"])?;
    if job_id==0 {return Err("missing job ID".into());}
    let runner=get_s(job,&["runner_name"])?;
    if runner.is_empty() {return Err("missing runner identity".into());}
    let steps=arr(job,"steps")?;
    let matching:Vec<_>=steps.iter().filter(|s|s["name"].as_str()==Some(&p.step_name)).collect();
    if matching.len()!=1 {return Err("required guest step missing/ambiguous".into());}
    eq(matching[0],&["status"],"completed")?;
    eq(matching[0],&["conclusion"],"success")?;

    let reviews=client.get(&format!("{base}/pulls/{}/reviews?per_page=100&page=1",p.pull_number))?;
    let review_list=reviews.as_array().ok_or("missing review list")?;
    if review_list.len()>=100 {return Err("review pagination incomplete".into());}
    let mut last_review=None;
    for review in review_list {
        let state=get_s(review,&["state"])?;
        let reviewer=get_s(review,&["user","login"])?;
        let sha=get_s(review,&["commit_id"])?;
        if state=="CHANGES_REQUESTED"&&sha==p.source_sha {
            return Err("blocking changes-requested review".into());
        }
        if reviewer.eq_ignore_ascii_case(&p.reviewer) {
            last_review=Some((state,sha));
        }
    }
    if last_review!=Some(("APPROVED",p.source_sha.as_str())) {
        return Err("current-head independent approval absent".into());
    }

    let protection=client.get(&format!("{base}/branches/main/protection"))?;
    if get_n(&protection,&["required_pull_request_reviews","required_approving_review_count"])?<1 {
        return Err("independent review not enforced on main".into());
    }
    let checks=protection.get("required_status_checks").ok_or("missing status policy")?;
    if checks["strict"].as_bool()!=Some(true) || arr(checks,"contexts")?.is_empty() {
        return Err("required status checks not enforced on main".into());
    }

    let artifacts=client.get(&format!("{run_path}/artifacts?per_page=100&page=1"))?;
    let all=arr(&artifacts,"artifacts")?;
    if get_n(&artifacts,&["total_count"])?!=all.len() as u64 {
        return Err("artifact pagination incomplete".into());
    }
    let matches:Vec<_>=all.iter().filter(|x|x["name"].as_str()==Some(&p.artifact_name)).collect();
    if matches.len()!=1 {return Err("expected guest receipt artifact missing".into());}
    let a=matches[0];
    if a["expired"].as_bool()!=Some(false) ||
       get_n(a,&["size_in_bytes"])?==0 ||
       get_n(a,&["workflow_run","id"])?!=p.run_id {
        return Err("expired, empty or unrelated artifact".into());
    }
    eq(a,&["workflow_run","head_sha"],&p.source_sha)?;
    let digest=get_s(a,&["digest"])?;
    if !digest.starts_with("sha256:")||digest.len()!=71||
        !digest.as_bytes()[7..].iter().all(|b|b.is_ascii_hexdigit()) {
        return Err("GitHub artifact digest metadata absent".into());
    }
    let artifact_id=get_n(a,&["id"])?;
    if artifact_id==0 {return Err("invalid artifact ID".into());}
    Ok(MetadataReceipt {
        repository:p.repository.clone(),run_id:p.run_id,
        head_sha:p.source_sha.clone(),tree_sha:p.tree_sha.clone(),
        job_id,runner_name:runner.into(),artifact_id,
        artifact_digest_claim:digest.into(),
        artifact_bytes_verified:false,guest_execution_verified:false,
    })
}
