//! Fail-closed source-only parity matrix for native Atto std.http imports.
//! This is a metadata consistency check, not compiled guest execution.
use serde_json::{json, Value};
use std::collections::BTreeSet;

const MANIFEST: &str = include_str!("../contracts/v1/http-namespace-import-parity-v1.json");
const BACKENDS: [&str; 11] = [
    "llvm", "java", "net", "wasm", "rs", "forge", "cranelift",
    "js", "atto", "hdl", "realtime",
];

fn valid(v: &Value) -> bool {
    if v["schema"] != "atto.feature.parity.std_http_namespace.v1"
        || v["status"] != "ALL_NOT_EXECUTED"
        || v["feature_id"] != "stdlib.http.namespace_import"
        || v["authority"] != "atto-lang/atto-platform-conformance/contracts/hosted-http-namespace-parity-v1.json"
        || v["canonical_import"] != r#"import * as http from "std.http";"#
        || v["selective_import"] != r#"import module http3 from "./library_public.atto";"#
        || v["public_module_declaration"] != "define pub module"
        || v["private_module_default"] != true
        || v["backend_owned_source"] != true
        || v["guest_required"] != true
    {
        return false;
    }
    let positives: BTreeSet<&str> = match v["required_positive"].as_array() {
        Some(items) if items.len() == 3 =>
            items.iter().filter_map(Value::as_str).collect(),
        _ => return false,
    };
    if positives != BTreeSet::from(["builtin_namespace", "selective_public_module", "public_module_wildcard"]) {
        return false;
    }
    let negatives: BTreeSet<&str> = match v["required_negative"].as_array() {
        Some(items) if items.len() == 5 =>
            items.iter().filter_map(Value::as_str).collect(),
        _ => return false,
    };
    if negatives != BTreeSet::from([
        "private_module", "private_member", "unregistered_builtin", "duplicate_alias",
        "public_member_private_module",
    ]) {
        return false;
    }
    let Some(backends) = v["backends"].as_array() else { return false; };
    if backends.len() != BACKENDS.len() { return false; }
    let mut seen = BTreeSet::new();
    for b in backends {
        let Some(name) = b["backend"].as_str() else { return false; };
        let expected_stage = match name {
            "llvm" => "SCALAR_NAMESPACE_AST_IMPORT_STAGED_UNVERIFIED",
            "java" => "PUBLIC_MODULE_VISIBILITY_STAGED_UNVERIFIED",
            "net" => "UNSUPPORTED_IMPORT_FAIL_CLOSED",
            _ => "UNASSESSED",
        };
        if !BACKENDS.contains(&name) || !seen.insert(name)
            || b["status"] != "NOT_EXECUTED"
            || b["guest_executed_cases"].as_u64() != Some(0)
            || b["source_stage"] != expected_stage
            || b.as_object().is_none_or(|m| m.len() != 4) {
            return false;
        }
    }
    seen.len() == BACKENDS.len() && v.as_object().is_some_and(|m| m.len() == 13)
}

#[test]
fn all_eleven_native_compilers_have_explicit_unexecuted_import_rows() {
    let v: Value = serde_json::from_str(MANIFEST).expect("valid JSON manifest");
    assert!(valid(&v), "missing, duplicated, forged or inconsistent backend row");
}

#[test]
fn false_guest_promotion_and_missing_negative_import_checks_are_rejected() {
    let original: Value = serde_json::from_str(MANIFEST).unwrap();
    for backend in BACKENDS {
        let mut altered = original.clone();
        let entry = altered["backends"].as_array_mut().unwrap().iter_mut()
            .find(|e| e["backend"] == backend).unwrap();
        entry["status"] = json!("PASS");
        assert!(!valid(&altered), "{backend}: false guest PASS");
    }
    let mut altered = original.clone();
    altered["backends"][10]["backend"] = json!("java");
    assert!(!valid(&altered), "duplicate backend");

    let mut altered = original.clone();
    altered["required_negative"].as_array_mut().unwrap().pop();
    assert!(!valid(&altered), "missing negative import case");

    let mut altered = original.clone();
    altered["guest_required"] = json!(false);
    assert!(!valid(&altered), "host-only simulation is not guest parity");

    for (backend, false_stage) in [
        ("llvm", "COMPILED_GUEST_PASS"),
        ("java", "SCALAR_NAMESPACE_AST_IMPORT_STAGED_UNVERIFIED"),
        ("net", "FULL_IMPORT_PARITY"),
    ] {
        let mut altered = original.clone();
        let row = altered["backends"].as_array_mut().unwrap().iter_mut()
            .find(|entry| entry["backend"] == backend).unwrap();
        row["source_stage"] = json!(false_stage);
        assert!(!valid(&altered), "{backend}: forged source stage");
    }

    let mut altered = original.clone();
    altered["backend_owned_source"] = json!(false);
    assert!(!valid(&altered), "backend-local .atto implementation must be preserved");
}
