use atto_evidence_contract_conformance::github_probe::{GithubReadOnly,ProbeSpec,probe_github_metadata};
use serde_json::{json,Value};
use std::collections::BTreeMap;
const SOURCE:&str="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE:&str="cccccccccccccccccccccccccccccccccccccccc";
const DIGEST:&str="dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const STEP:&str="Run cargo +1.88.0 test --locked --all-targets --verbose";
struct Mock{values:BTreeMap<String,Value>}
impl GithubReadOnly for Mock{
    fn get_json(&self,p:&str)->Result<Value,String>{
        self.values.get(p).cloned().ok_or_else(||format!("GitHub 403/404: {p}"))
    }
}
fn key(p:&str)->String{format!("/repos/benefactor-cc/ci/{p}")}
fn spec<'a>() -> ProbeSpec<'a>{
    ProbeSpec{repository:"benefactor-cc/ci",pr_number:5,run_id:42,source_sha:SOURCE,
        source_tree_sha:TREE,workflow_path:".github/workflows/conformance.yml",
        job_name:"guest-tests",required_steps:&[STEP],approved_reviewer:"ORESoftware",
        artifact_name:"guest-receipt"}
}
fn fixture()->Mock{
    let mut v=BTreeMap::new();
    v.insert(key("pulls/5"),json!({"state":"open","draft":false,
        "head":{"sha":SOURCE},"base":{"ref":"main"},"user":{"login":"the1mills"}}));
    v.insert(key(&format!("git/commits/{SOURCE}")),json!({"sha":SOURCE,"tree":{"sha":TREE}}));
    v.insert(key("actions/runs/42"),json!({"id":42,"run_attempt":2,"head_sha":SOURCE,
        "repository":{"full_name":"benefactor-cc/ci"},"event":"pull_request",
        "path":".github/workflows/conformance.yml","status":"completed","conclusion":"success"}));
    v.insert(key("actions/runs/42/jobs?per_page=100"),json!({"total_count":1,
        "jobs":[{"id":123,"run_id":42,"run_attempt":2,"head_sha":SOURCE,"name":"guest-tests",
            "status":"completed","conclusion":"success",
            "steps":[{"name":STEP,"status":"completed","conclusion":"success"}]}]}));
    v.insert(key("pulls/5/reviews?per_page=100"),json!([
        {"id":321,"user":{"login":"ORESoftware"},"state":"APPROVED","commit_id":SOURCE}
    ]));
    v.insert(key("branches/main/protection"),json!({
        "required_pull_request_reviews":{"required_approving_review_count":1},
        "required_status_checks":{"strict":true},"enforce_admins":{"enabled":true}
    }));
    v.insert(key("actions/runs/42/artifacts?per_page=100"),json!({"total_count":1,
        "artifacts":[{"id":400,"name":"guest-receipt","expired":false,
            "workflow_run":{"id":42},"digest":format!("sha256:{DIGEST}")}]}));
    Mock{values:v}
}
fn ok(m:&Mock)->bool{probe_github_metadata(m,&spec()).is_ok()}
fn change(f:impl FnOnce(&mut Value),path:&str)->bool{
    let mut m=fixture();f(m.values.get_mut(&key(path)).unwrap());ok(&m)
}
#[test] fn successful_metadata_is_never_release_certification(){
    let x=probe_github_metadata(&fixture(),&spec()).unwrap();
    assert_eq!(x.job_id,123);assert_eq!(x.artifact_id,400);
    assert_eq!(x.release_certification,"NOT_CERTIFIED");
    assert!(!x.artifact_bytes_verified && !x.executed_guest_verified);
}
#[test] fn rejects_stale_run_incomplete_job_and_pretend_execution(){
    for (p,field,value) in [
        ("actions/runs/42","head_sha",json!("ffffffffffffffffffffffffffffffffffffffff")),
        ("actions/runs/42","status",json!("queued")),
        ("actions/runs/42","conclusion",json!("skipped")),
        ("actions/runs/42","event",json!("workflow_dispatch")),
        ("actions/runs/42","path",json!(".github/workflows/evil.yml")),
    ]{
        assert!(!change(|v|v[field]=value,p),"{p}:{field}");
    }
    let p="actions/runs/42/jobs?per_page=100";
    assert!(!change(|v|v["jobs"][0]["run_attempt"]=json!(1),p));
    assert!(!change(|v|v["jobs"][0]["conclusion"]=json!("success-but-skipped"),p));
    assert!(!change(|v|v["jobs"][0]["steps"][0]["conclusion"]=json!("skipped"),p));
    assert!(!change(|v|v["jobs"][0]["steps"][0]["name"]=json!("unrelated"),p));
    assert!(!change(|v|v["total_count"]=json!(101),p));
}
#[test] fn rejects_reviewer_spoof_and_protection_absence(){
    let p="pulls/5/reviews?per_page=100";
    assert!(!change(|v|v[0]["commit_id"]=json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),p));
    assert!(!change(|v|v[0]["user"]["login"]=json!("the1mills"),p));
    assert!(!change(|v|v[0]["state"]=json!("DISMISSED"),p));
    assert!(!change(|v|v[0]["state"]=json!("COMMENTED"),p));
    for (name,value) in [("required_status_checks",json!({"strict":false})),
        ("required_pull_request_reviews",json!({"required_approving_review_count":0})),
        ("enforce_admins",json!({"enabled":false}))]{
        assert!(!change(|v|v[name]=value,"branches/main/protection"));
    }
    let mut m=fixture();m.values.remove(&key("branches/main/protection"));
    assert!(!ok(&m),"403 branch protection must block");
}
#[test] fn rejects_artifact_claims_without_matching_metadata(){
    let p="actions/runs/42/artifacts?per_page=100";
    assert!(!change(|v|v["total_count"]=json!(2),p));
    assert!(!change(|v|v["artifacts"][0]["expired"]=json!(true),p));
    assert!(!change(|v|v["artifacts"][0]["name"]=json!("other"),p));
    assert!(!change(|v|v["artifacts"][0]["workflow_run"]["id"]=json!(43),p));
    assert!(!change(|v|v["artifacts"][0]["digest"]=json!("sha256:deadbeef"),p));
    assert!(!change(|v|v["artifacts"]=json!([]),p));
}
#[test] fn rejects_wrong_pr_tree_and_invalid_client_inputs(){
    assert!(!change(|v|v["head"]["sha"]=json!("f".repeat(40)),"pulls/5"));
    assert!(!change(|v|v["draft"]=json!(true),"pulls/5"));
    assert!(!change(|v|v["user"]["login"]=json!("ORESoftware"),"pulls/5"));
    assert!(!change(|v|v["tree"]["sha"]=json!("f".repeat(40)),
        &format!("git/commits/{SOURCE}")));
    for bad in ["owner/repo/extra","a/..","evil.com/@repo","owner/repo?x=1"]{
        let p=ProbeSpec{repository:bad,..spec()};
        assert!(probe_github_metadata(&fixture(),&p).is_err());
    }
}
