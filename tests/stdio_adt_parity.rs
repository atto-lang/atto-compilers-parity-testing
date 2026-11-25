//! Portable checked stdio + nominal on-is ADT contract admission.
 //! This test does NOT run a compiler or claim any backend has passed.
use serde_json::{json, Value};
use std::collections::BTreeSet;
const MANIFEST: &str = include_str!("../contracts/v1/stdio-adt-parity-v1.json");
const BACKENDS: [&str; 11] = [
    "java","llvm","forge","net","wasm","rs","js","cranelift",
    "atto","hdl","realtime",
];
const GUESTS: [&str; 4] = [
    "checked_echo","checked_echo_readch_alias",
    "checked_echo_is_variants","checked_echo_is_variants_readch",
];
const NEGATIVES: [&str; 13] = [
    "is_unknown_extractor","is_none_payload","is_void_ok_bound",
    "is_missing_ok_payload","is_wrong_variant_family","is_nested_extractor",
    "is_multi_payload","is_duplicate_arm","unchecked_read_payload",
    "wrong_read_direction","wrong_write_direction","read_without_suspend",
    "stream_and_readch_conflict",
];

fn valid(v: &Value) -> bool {
    if v["schema"]!="atto.feature.parity.stdio.is_adt.v1"
        || v["feature_id"]!="stdio.checked_string_adt_is_pattern"
        || v["status"]!="ALL_NOT_EXECUTED"
        || v["guest_required"]!=true
        || v["allow_host_only_as_pass"]!=false
        || v["require_exact_source_head"]!=true
        || v["require_unmodified_fixture_head"]!=true
        || v["hdl_profile"].as_str().is_none_or(|x|!x.contains("E_UNSUPPORTED_PROFILE"))
        || v["runner"]!="tools/run-stdio-conformance.mjs" {
        return false;
    }
    let Some(backends)=v["backends"].as_array() else{return false;};
    if backends.len()!=BACKENDS.len(){return false;}
    let mut seen=BTreeSet::new();
    for backend in backends {
        let Some(name)=backend["backend"].as_str() else {return false;};
        if !BACKENDS.contains(&name)||!seen.insert(name)
            || backend["status"]!="NOT_EXECUTED"
            || backend["guest_executed_cases"]!=0
            || backend.as_object().is_none_or(|m|m.len()!=3) {
            return false;
        }
    }
    let match_ids=|field:&str,expected:&[&str]|{
        let Some(items)=v[field].as_array() else{return false;};
        let seen: BTreeSet<&str>=items.iter().filter_map(Value::as_str).collect();
        seen.len()==expected.len()&&expected.iter().all(|x|seen.contains(x))
    };
    match_ids("required_guest_fixtures",&GUESTS)
      &&match_ids("security_negative_gates",&NEGATIVES)
      &&match_ids("additional_compile_fixtures",&["stdio_is_pattern_fallback"])
      &&match_ids("accepted_is_adt_aliases",&[
        "on is Ok<_>","on is Ok<line>","on is Some<item>",
        "on is Err<error>","on is None"])
}

#[test]
fn all_compilers_are_accounted_for_without_fake_guest_parity() {
    let value: Value=serde_json::from_str(MANIFEST).expect("stdio matrix JSON");
    assert!(valid(&value),"missing or inconsistent cross-compiler stdio matrix");
}

#[test]
fn forged_pass_missing_negatives_and_host_only_approvals_fail_closed() {
    let original:Value=serde_json::from_str(MANIFEST).unwrap();
    for name in BACKENDS {
        let mut test=original.clone();
        test["backends"].as_array_mut().unwrap().iter_mut()
            .find(|x|x["backend"]==name).unwrap()["status"]=json!("PASS");
        assert!(!valid(&test),"{name} forged PASS");
    }
    for (field,value) in [
        ("guest_required",json!(false)),
        ("allow_host_only_as_pass",json!(true)),
        ("require_exact_source_head",json!(false)),
        ("status",json!("PASS")),
    ] {
        let mut test=original.clone();test[field]=value;
        assert!(!valid(&test),"failed to reject forged {field}");
    }
    for field in ["required_guest_fixtures","security_negative_gates",
                  "accepted_is_adt_aliases"] {
        let mut test=original.clone();test[field].as_array_mut().unwrap().pop();
        assert!(!valid(&test),"removed {field} must fail");
    }
    let mut duplicate=original.clone();
    duplicate["backends"][10]["backend"]=json!("rs");
    assert!(!valid(&duplicate),"duplicate backend accepted");
}

#[test]
fn hdl_is_unsupported_not_a_emulated_stdin_channel() {
    let value:Value=serde_json::from_str(MANIFEST).unwrap();
    assert!(value["hdl_profile"].as_str().unwrap().contains("E_UNSUPPORTED_PROFILE"));
    assert!(value["wasi_profile"].as_str().unwrap().contains("default deny"));
    assert_eq!(value["guest_required"],true);
}
