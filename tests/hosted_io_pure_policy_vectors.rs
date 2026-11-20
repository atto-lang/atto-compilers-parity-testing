//! Independent Rust reference oracle for hosted std pure-policy vectors.
 //! No compiler or Atto guest executes in this test: SOURCE_ONLY evidence.
use serde_json::Value;
use std::collections::BTreeSet;

const VECTORS: &str = include_str!("../contracts/v1/hosted-io-pure-policy-vectors-v1.json");
const PARITY: &str = include_str!("../contracts/v1/async-http-fs-parity-v1.json");
const HTTP_SHA: &str = "22757dafe89bc9601d04c8726a09ddf676206c7e";
const FS_SHA: &str = "0cd53216e91ef8de1a764ba5b80ce85c5796c1f2";
const OPS: [&str; 9] = [
    "http_status_valid", "http_status_success", "http_status_redirect",
    "http_status_client_error", "http_status_server_error",
    "http_get_status_may_have_body", "fs_text_size_admissible",
    "fs_text_range_admissible", "fs_text_replacement_admissible",
];

fn reference(op: &str, a: &[i64]) -> Option<bool> {
    match (op, a) {
        ("http_status_valid", &[s]) => Some((100..=599).contains(&s)),
        ("http_status_success", &[s]) => Some((200..=299).contains(&s)),
        ("http_status_redirect", &[s]) => Some((300..=399).contains(&s)),
        ("http_status_client_error", &[s]) => Some((400..=499).contains(&s)),
        ("http_status_server_error", &[s]) => Some((500..=599).contains(&s)),
        ("http_get_status_may_have_body", &[s]) =>
            Some((200..=599).contains(&s) && ![204, 205, 304].contains(&s)),
        ("fs_text_size_admissible", &[bytes, file_limit, available]) =>
            Some(bytes >= 0 && file_limit >= 0 && available >= 0 &&
                bytes <= file_limit && bytes <= available),
        ("fs_text_range_admissible", &[offset, len, size]) =>
            Some(offset >= 0 && len >= 0 && size >= 0 &&
                offset <= size && len <= size - offset.max(0).min(size)),
        ("fs_text_replacement_admissible", &[old, new, used, total]) => {
            if old < 0 || new < 0 || used < 0 || total < 0 ||
                old > used || used > total || new > total {
                return Some(false);
            }
            Some(new <= total - (used - old))
        }
        _ => None,
    }
}

fn contract_ok(v: &Value) -> bool {
    if v["schema"] != "atto.stdlib.hosted_io_pure_policy_vectors.v1" ||
        v["status"] != "REFERENCE_ORACLE_NOT_COMPILED" ||
        v["backend_guest_status"] != "NOT_EXECUTED" ||
        v["scalar_type"] != "signed-i64-decimal-string" ||
        v["source_blobs"]["stdlib/std/http_response_policy.atto"] != HTTP_SHA ||
        v["source_blobs"]["stdlib/std/fs_text_policy.atto"] != FS_SHA {
        return false;
    }
    let Some(cases) = v["cases"].as_array() else { return false; };
    if cases.len() != 56 { return false; }
    let mut unique = BTreeSet::new();
    let mut seen_ops = BTreeSet::new();
    for case in cases {
        let Some(id) = case["id"].as_str() else { return false; };
        let Some(op) = case["operation"].as_str() else { return false; };
        let Some(args) = case["args"].as_array() else { return false; };
        let Some(expected) = case["expected"].as_bool() else { return false; };
        if !id.starts_with(&format!("{op}.")) || !unique.insert(id) ||
            !OPS.contains(&op) || !seen_ops.insert(op) && !OPS.contains(&op) ||
            case.as_object().is_none_or(|obj| obj.len() != 4) {
            return false;
        }
        let parsed = args.iter().map(|x| x.as_str().and_then(|s|s.parse::<i64>().ok()))
            .collect::<Option<Vec<_>>>();
        let Some(values) = parsed else { return false; };
        if reference(op, &values) != Some(expected) { return false; }
    }
    seen_ops.len() == OPS.len()
}

#[test]
fn all_fifty_six_vectors_match_i64_reference_without_claiming_guest_execution() {
    let v: Value = serde_json::from_str(VECTORS).unwrap();
    assert!(contract_ok(&v), "vector source or expected semantics drifted");
    let parity: Value = serde_json::from_str(PARITY).unwrap();
    assert_eq!(parity["status"], "ALL_NOT_EXECUTED");
    assert!(parity["backends"].as_array().unwrap().iter().all(|x|
        x["status"] == "NOT_EXECUTED" && x["guest_executed_cases"] == 0
    ));
}

#[test]
fn false_guest_status_mutated_vectors_and_unsafe_arithmetic_fail_closed() {
    let input: Value = serde_json::from_str(VECTORS).unwrap();
    let mut changed = input.clone();
    changed["status"] = serde_json::json!("PASS");
    assert!(!contract_ok(&changed));
    let mut changed = input.clone();
    changed["cases"][0]["expected"] = serde_json::json!(true);
    assert!(!contract_ok(&changed));
    let mut changed = input.clone();
    changed["cases"][1]["args"][0] = serde_json::json!("9223372036854775808");
    assert!(!contract_ok(&changed));
    let mut changed = input.clone();
    changed["backend_guest_status"] = serde_json::json!("PASS");
    assert!(!contract_ok(&changed));
    assert_eq!(reference("fs_text_range_admissible",
        &[i64::MAX-1,2,i64::MAX]), Some(false));
    assert_eq!(reference("fs_text_replacement_admissible",
        &[i64::MAX-1,i64::MAX,i64::MAX,i64::MAX]), Some(false));
    assert_eq!(reference("fs_text_replacement_admissible",
        &[i64::MAX,i64::MAX,i64::MAX,i64::MAX]), Some(true));
}
