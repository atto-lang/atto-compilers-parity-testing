use atto_evidence_contract_conformance::github_inspect::{
    inspect,CurlGitHub,Expected,GitHubReader
};
use atto_evidence_contract_conformance::decode_strict_json;
use serde_json::{json,Value};
use std::collections::BTreeMap;

struct MockGitHub { responses: BTreeMap<String, Value> }
impl GitHubReader for MockGitHub {
    fn get(&self,path:&str)->Result<Value,String> {
        self.responses.get(path).cloned().ok_or_else(||format!("API unavailable: {path}"))
    }
}
const REPO:&str="benefactor-cc/atto-guest-ci";
const RUN:&str="/repos/benefactor-cc/atto-guest-ci/actions/runs/42";
const BASE:&str="/repos/benefactor-cc/atto-guest-ci";
fn fixtures()->(Expected,MockGitHub) {
    let sha="a".repeat(40);
    let tree="b".repeat(40);
    let expectation=Expected{
        repository:REPO.into(),pull_number:1,run_id:42,
        source_sha:sha.clone(),tree_sha:tree.clone(),
        job_name:"guest-fixtures".into(),step_name:"execute actual guest".into(),
        artifact_name:"guest-receipt".into(),reviewer:"ORESoftware".into(),
    };
    let mut api=BTreeMap::new();
    api.insert(RUN.into(),json!({
        "id":42,"run_attempt":1,"repository":{"full_name":REPO},
        "head_sha":sha,"status":"completed","conclusion":"success"
    }));
    api.insert(format!("{BASE}/git/commits/{sha}"),json!({
        "sha":sha,"tree":{"sha":tree}
    }));
    api.insert(format!("{BASE}/pulls/1"),json!({
        "state":"open","draft":false,"head":{"sha":sha},
        "base":{"ref":"main"},"user":{"login":"the1mills"}
    }));
    api.insert(format!("{RUN}/jobs?per_page=100&page=1&filter=latest"),json!({
        "total_count":1,"jobs":[{"id":20,"run_id":42,"run_attempt":1,
            "name":"guest-fixtures","runner_name":"GitHub Actions 1",
            "status":"completed","conclusion":"success",
            "steps":[{"name":"execute actual guest","status":"completed","conclusion":"success"}]
        }]
    }));
    api.insert(format!("{BASE}/pulls/1/reviews?per_page=100&page=1"),json!([
        {"user":{"login":"ORESoftware"},"state":"APPROVED","commit_id":sha}
    ]));
    api.insert(format!("{BASE}/branches/main/protection"),json!({
        "required_pull_request_reviews":{"required_approving_review_count":1},
        "required_status_checks":{"strict":true,"contexts":["guest-fixtures"]}
    }));
    api.insert(format!("{RUN}/artifacts?per_page=100&page=1"),json!({
        "total_count":1,"artifacts":[{
            "id":66,"name":"guest-receipt","expired":false,"size_in_bytes":1024,
            "digest":format!("sha256:{}", "d".repeat(64)),
            "workflow_run":{"id":42,"head_sha":sha}
        }]
    }));
    (expectation,MockGitHub{responses:api})
}
fn denied(p:&Expected,api:&MockGitHub) { assert!(inspect(api,p).is_err()); }
#[test]
fn verifies_exact_api_metadata_but_never_certifies_guest_or_artifact_bytes() {
    let (p,a)=fixtures();
    let o=inspect(&a,&p).expect("valid synthetic GitHub metadata must parse");
    assert_eq!(o.repository,REPO);
    assert_eq!(o.job_id,20);
    assert_eq!(o.artifact_id,66);
    assert_eq!(o.tree_sha,p.tree_sha);
    assert!(!o.artifact_bytes_verified);
    assert!(!o.guest_execution_verified);
}
#[test]
fn reject_stale_or_forged_head_tree_and_run_metadata() {
    let (mut p,api)=fixtures();
    p.source_sha="e".repeat(40);denied(&p,&api);
    let (mut p,api)=fixtures();p.tree_sha="e".repeat(40);denied(&p,&api);
    let (mut p,api)=fixtures();p.run_id=43;denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(RUN).unwrap()["conclusion"]=json!("failure");denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(RUN).unwrap()["status"]=json!("in_progress");denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(RUN).unwrap()["run_attempt"]=json!(0);denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(RUN).unwrap()["repository"]["full_name"]=json!("attacker/repo");denied(&p,&api);
}
#[test]
fn reject_skipped_or_wrong_job_step_and_pagination() {
    let path=format!("{RUN}/jobs?per_page=100&page=1&filter=latest");
    let (p,mut api)=fixtures();api.responses.get_mut(&path).unwrap()["total_count"]=json!(2);denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(&path).unwrap()["jobs"][0]["conclusion"]=json!("skipped");denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(&path).unwrap()["jobs"][0]["runner_name"]=json!("");denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(&path).unwrap()["jobs"][0]["run_attempt"]=json!(2);denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(&path).unwrap()["jobs"][0]["steps"][0]["conclusion"]=json!("skipped");denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(&path).unwrap()["jobs"][0]["steps"][0]["name"]=json!("host-only");denied(&p,&api);
    let (p,mut api)=fixtures();api.responses.get_mut(&path).unwrap()["jobs"].as_array_mut().unwrap().clear();denied(&p,&api);
}
#[test]
fn reject_unauthorized_stale_and_dismissed_reviews_and_branch_policy() {
    let review=format!("{BASE}/pulls/1/reviews?per_page=100&page=1");
    let prot=format!("{BASE}/branches/main/protection");
    let pull=format!("{BASE}/pulls/1");
    let (p,mut a)=fixtures();a.responses.get_mut(&review).unwrap().as_array_mut().unwrap().clear();denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&review).unwrap()[0]["state"]=json!("DISMISSED");denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&review).unwrap()[0]["commit_id"]=json!("e".repeat(40));denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&review).unwrap()[0]["user"]["login"]=json!("the1mills");denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&pull).unwrap()["user"]["login"]=json!("ORESoftware");denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&pull).unwrap()["draft"]=json!(true);denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&prot).unwrap()["required_pull_request_reviews"]["required_approving_review_count"]=json!(0);denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&prot).unwrap()["required_status_checks"]["strict"]=json!(false);denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.remove(&prot);denied(&p,&a);
}
#[test]
fn reject_missing_expired_or_unverified_artifact_metadata() {
    let path=format!("{RUN}/artifacts?per_page=100&page=1");
    let (p,mut a)=fixtures();a.responses.get_mut(&path).unwrap()["total_count"]=json!(3);denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&path).unwrap()["artifacts"][0]["expired"]=json!(true);denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&path).unwrap()["artifacts"][0]["size_in_bytes"]=json!(0);denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&path).unwrap()["artifacts"][0]["digest"]=json!("sha256:000");denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&path).unwrap()["artifacts"][0]["workflow_run"]["id"]=json!(41);denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&path).unwrap()["artifacts"][0]["workflow_run"]["head_sha"]=json!("e".repeat(40));denied(&p,&a);
    let (p,mut a)=fixtures();a.responses.get_mut(&path).unwrap()["artifacts"].as_array_mut().unwrap().clear();denied(&p,&a);
}
#[test]
fn rejects_untrusted_origins_and_malformed_transport_credentials() {
    for s in ["evil.com/repo/x","evil@github.com/repo","/repo","owner/..",
              "owner/repo?injection=1","https://github.com/x/y","bad/repo\n"] {
        let (mut p,a)=fixtures();p.repository=s.into();denied(&p,&a);
    }
    assert!(CurlGitHub::new("bad\nX-Header: evil".into()).is_err());
    assert!(CurlGitHub::new("bad\"header".into()).is_err());
    assert!(CurlGitHub::new("token".into()).is_ok());
    use atto_evidence_contract_conformance::github_inspect::GitHubReader;
    let c=CurlGitHub::new("token".into()).unwrap();
    for path in ["https://evil.example","/repos/foo/bar/../../secrets",
                 "/repos/foo/bar?x=%2f", "/users/the1mills"] {
        assert!(c.get(path).is_err(),"{path}");
    }
}
#[test]
fn strict_github_json_decoder_rejects_ambiguous_payloads() {
    for raw in [br#"{"a":1,"a":2}"#.as_slice(),b"{",b"\xff",
        br#"{"jobs":[{"id":1,"id":2}]}"#,br#"{"x":NaN}"#,br#"{"x":1.0}"#,
        br#"{"x":"\ud800"}"#] {
        assert!(decode_strict_json(raw).is_err(),"{raw:?}");
    }
    assert!(decode_strict_json(br#"[{"id":1}]"#).unwrap().is_array());
}
