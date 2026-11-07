use atto_evidence_contract_conformance::{decode_and_validate, typespec_fields, wit_fields, MAX_WIRE_BYTES};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const SCHEMA: &str = include_str!("../contracts/v1/conformance-evidence.schema.json");
const TSP: &str = include_str!("../contracts/v1/conformance-evidence.tsp");
const WIT: &str = include_str!("../contracts/v1/conformance-evidence.wit");

fn schema() -> Value { serde_json::from_str(SCHEMA).unwrap() }
fn base() -> Value { json!({"version":1,"backend":"llvm","feature_id":"actor.ownership",
    "source_sha":"a".repeat(40),"case_set_sha256":"b".repeat(64)}) }
fn passing() -> Value {let mut v=base();v["status"]=json!("passed");v["execution_target"]=json!("guest");
    v["run_url"]=json!("https://github.com/benefactor-cc/ci/actions/runs/42");
    v["executed_cases"]=json!(5);v["failed_cases"]=json!(0);v }
fn failing() -> Value {let mut v=passing();v["status"]=json!("failed");v["failed_cases"]=json!(2);v["reason"]=json!("exit mismatch");v}
fn skipped() -> Value {let mut v=base();v["status"]=json!("not_executed");v["reason"]=json!("runner unavailable");v}
fn unsupported() -> Value {let mut v=base();v["status"]=json!("unsupported");v["reason"]=json!("no implementation");v}
fn check(v: &Value) -> Result<Value,String> {
    decode_and_validate(v.to_string().as_bytes(),&schema()).and_then(|value| {
        if ["passed","failed"].contains(&value["status"].as_str().unwrap_or("")) &&
            value["failed_cases"].as_u64().unwrap_or(0) > value["executed_cases"].as_u64().unwrap_or(0) {
            return Err("failed_cases exceeds executed_cases".into());
        }
        Ok(value)
    })
}
fn reject(v: Value) {assert!(check(&v).is_err(),"accepted invalid conformance evidence: {v}");}

#[test]
fn all_four_statuses_validate_structurally() {
    assert!(jsonschema::draft202012::meta::is_valid(&schema()));
    for v in [passing(),failing(),skipped(),unsupported()] {assert_eq!(check(&v).unwrap(),v);}
}
#[test]
fn reject_false_passes_and_missing_provenance_fields() {
    for (field,value) in [
        ("status",json!("pending")),("status",json!("skipped")),("status",json!("unknown")),
        ("failed_cases",json!(1)),("execution_target",json!("simulated")),("executed_cases",json!(0)),
        ("run_url",json!("https://example.org/runs/12")),("run_url",json!("http://github.com/a/b/actions/runs/42")),
        ("run_url",json!("https://github.com/a/b/actions/runs/42\n")),
        ("source_sha",json!("a".repeat(39))),("source_sha",json!(format!("{}\n","a".repeat(40)))),
        ("case_set_sha256",json!("b".repeat(65))),("case_set_sha256",json!("z".repeat(64))),
        ("feature_id",json!("not allowed")),("reason",json!("unsolicited")),
    ] {
        let mut v=passing();v[field]=value;reject(v);
    }
    for field in ["status","source_sha","case_set_sha256","execution_target","run_url","executed_cases","failed_cases"] {
        let mut v=passing();v.as_object_mut().unwrap().remove(field);reject(v);
    }
    let mut v=failing();v["failed_cases"]=json!(0);reject(v);
    let mut v=failing();v["failed_cases"]=json!(6);reject(v); // cross-field semantic check
    let mut v=skipped();v["run_url"]=json!("https://github.com/a/b/actions/runs/42");reject(v);
    let mut v=unsupported();v["executed_cases"]=json!(1);reject(v);
}
#[test]
fn rejects_type_confusion_negative_matrix() {
    let invalid=[json!(null),json!(true),json!(false),json!(123),json!(1.5),json!([]),json!({})];
    let mut n=0;
    for field in ["backend","feature_id","source_sha","case_set_sha256","status",
                  "execution_target","run_url","executed_cases","version"] {
        for replacement in &invalid {
            let mut v=passing();v[field]=replacement.clone();reject(v);n+=1;
        }
    }
    for replacement in [json!(null),json!(true),json!(false),json!("one"),json!(1.5),json!([]),json!({})] {
        let mut v=passing();v["failed_cases"]=replacement;reject(v);n+=1;
    }
    for replacement in [json!(null),json!(true),json!(123),json!(1.5),json!([]),json!({}),json!("")] {
        let mut v=failing();v["reason"]=replacement;reject(v);n+=1;
    }
    assert!(n>=75);
}
#[test]
fn reject_duplicate_properties_malformed_utf8_and_noncanonical_numbers() {
    let v=passing();let raw=v.to_string();
    for name in v.as_object().unwrap().keys() {
        let input=format!("{},\"{}\":{}}}",&raw[..raw.len()-1],name,v[name]);
        assert!(decode_and_validate(input.as_bytes(),&schema()).is_err());
    }
    for invalid in [b"\xff".as_slice(),b"{}",b"null",b"{",b"[]",
        br#"{"a":{"x":1,"x":2}}"#,br#"{"version":1.0}"#,br#"{"version":1e0}"#,
        br#"{"a":NaN}"#,br#"{"feature_id":"\ud800"}"#,br#"{"a":1}{"b":2}"#] {
        assert!(decode_and_validate(invalid,&schema()).is_err(),"{invalid:?}");
    }
    assert!(decode_and_validate(&vec![b' '; MAX_WIRE_BYTES+1],&schema()).is_err());
}
#[test]
fn wit_and_typespec_field_types_align_with_schema() {
    let expected: BTreeMap<String,String>=[
        ("version","u32"),("backend","backend"),("feature-id","string"),("source-sha","string"),
        ("case-set-sha256","string"),("status","evidence-status"),
        ("execution-target","option<execution-target>"),("run-url","option<string>"),
        ("executed-cases","option<u32>"),("failed-cases","option<u32>"),("reason","option<string>")
    ].into_iter().map(|(k,v)|(k.into(),v.into())).collect();
    assert_eq!(wit_fields(WIT,"conformance-evidence"),expected);
    let fields: BTreeSet<_>=schema()["properties"].as_object().unwrap().keys().cloned().collect();
    assert_eq!(fields,expected.keys().map(|x|x.replace('-',"_")).collect());
    assert_ne!(wit_fields(&WIT.replace("executed-cases: option<u32>","executed-cases: u64"),"conformance-evidence"),expected);
    let types=typespec_fields(TSP);
    for (field,ty) in [
        ("version","1"),("backend","Backend"),("feature_id","string"),("source_sha","string"),
        ("case_set_sha256","string"),("execution_target","ExecutionTarget"),("run_url","url"),
        ("executed_cases","uint32"),("reason","string")] {
        assert_eq!(types[field],BTreeSet::from([ty.to_string()]));
    }
    assert_eq!(types["failed_cases"],BTreeSet::from(["0".into(),"uint32".into()]));
    let statuses: BTreeSet<_>=["passed","failed","unsupported","not_executed"].into_iter()
        .map(|x|format!("\"{x}\"")).collect();
    assert_eq!(types["status"],statuses);
}
#[test]
fn test_report_is_not_attestation_and_execution_source_must_be_external() {
    assert!(WIT.contains("enum execution-target { host, guest }"));
    assert!(WIT.contains("not-executed"));
    assert!(TSP.contains("@pattern("));
    let schema=schema();
    let variants: BTreeSet<_>=schema["oneOf"].as_array().unwrap().iter()
        .map(|a|a["properties"]["status"]["const"].as_str().unwrap()).collect();
    assert_eq!(variants,BTreeSet::from(["passed","failed","unsupported","not_executed"]));
    // A fabricated but well-shaped action URL is accepted by a schema.
    // Do not turn it into release proof without read-only GitHub attestation.
    assert!(check(&passing()).is_ok());
}
