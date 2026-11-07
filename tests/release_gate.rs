use atto_evidence_contract_conformance::release_gate::{
    accept_verified_passing_evidence, ObservedRun, ReleasePolicy,
};
use serde_json::{json,Value};
const SCHEMA: &str = include_str!("../contracts/v1/conformance-evidence.schema.json");
const SOURCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CASES: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
fn schema() -> Value { serde_json::from_str(SCHEMA).unwrap() }
fn policy() -> ReleasePolicy<'static> {
    ReleasePolicy {
        trusted_repository:"benefactor-cc/ci", source_sha:SOURCE, case_set_sha256:CASES,
        backend:"llvm", feature_id:"actor.ownership", require_guest:true,
    }
}
fn report() -> Value {
    json!({"version":1,"backend":"llvm","feature_id":"actor.ownership",
        "source_sha":SOURCE,"case_set_sha256":CASES,"status":"passed",
        "execution_target":"guest","run_url":"https://github.com/benefactor-cc/ci/actions/runs/42",
        "executed_cases":7,"failed_cases":0})
}
fn observation() -> ObservedRun<'static> {
    ObservedRun {
        repository:"benefactor-cc/ci",run_id:42,head_sha:SOURCE,
        complete:true,workflow_conclusion:"success",test_job_conclusion:"success",
        test_steps_executed:true,execution_target:"guest",executed_cases:7,
        failed_cases:0,case_set_sha256:CASES,artifact_digests_verified:true,
        independent_reviews_verified:true,protected_main_verified:true,
    }
}
fn attempt(report: &Value, policy: &ReleasePolicy<'_>, observed: &ObservedRun<'_>) -> bool {
    accept_verified_passing_evidence(report.to_string().as_bytes(),&schema(),policy,observed).is_ok()
}
#[test]
fn accepts_only_consistent_synthetic_positive_observation() {
    // A passing synthetic vector proves only this offline predicate works.
    let result=accept_verified_passing_evidence(report().to_string().as_bytes(),
        &schema(),&policy(),&observation()).unwrap();
    assert_eq!(result.executed_cases,7);
    assert_eq!(result.github_run,"https://github.com/benefactor-cc/ci/actions/runs/42");
}
#[test]
fn rejects_report_mutations_even_when_json_schema_would_accept() {
    for (field,bad) in [
        ("status",json!("failed")),("status",json!("unsupported")),
        ("status",json!("not_executed")),("backend",json!("wasm")),
        ("feature_id",json!("actor.other")),("source_sha",json!("c".repeat(40))),
        ("case_set_sha256",json!("d".repeat(64))),("execution_target",json!("host")),
        ("run_url",json!("https://github.com/evil/ci/actions/runs/42")),
        ("run_url",json!("https://github.com/benefactor-cc/ci/actions/runs/43")),
        ("executed_cases",json!(6)),("executed_cases",json!(8)),("failed_cases",json!(1)),
    ] {
        let mut r=report();r[field]=bad;
        assert!(!attempt(&r,&policy(),&observation()),"{field}: {r}");
    }
}
#[test]
fn rejects_untrusted_or_stale_observation_mutations() {
    let r=report();
    let mut o=observation();o.repository="evil/ci";assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.run_id=43;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.run_id=0;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.head_sha="f";assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.case_set_sha256="f";assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.complete=false;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.workflow_conclusion="pending";assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.workflow_conclusion="failure";assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.test_job_conclusion="skipped";assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.test_steps_executed=false;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.execution_target="host";assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.executed_cases=0;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.executed_cases=8;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.failed_cases=1;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.artifact_digests_verified=false;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.independent_reviews_verified=false;assert!(!attempt(&r,&policy(),&o));
    let mut o=observation();o.protected_main_verified=false;assert!(!attempt(&r,&policy(),&o));
}
#[test]
fn rejects_untrusted_repository_syntax_and_host_only_runs() {
    let r=report();
    for invalid in ["evil/repo/extra","bad..owner/repo","evil@github.com/repo",
        "evil/repo?foo=x","example.com\\@evil/repo","/repo","owner/..","owner/name#x"] {
        let p=ReleasePolicy { trusted_repository:invalid,..policy() };
        assert!(!attempt(&r,&p,&observation()),"{invalid}");
    }
    let p=ReleasePolicy{ require_guest:false, ..policy() };
    assert!(attempt(&r,&p,&observation()));
    let mut host=report();host["execution_target"]=json!("host");
    let mut o=observation();o.execution_target="host";
    assert!(!attempt(&host,&policy(),&o));
    assert!(attempt(&host,&p,&o)); // Non-guest policy must be explicit.
}
