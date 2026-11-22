//! Source-only parity metadata gate. Do NOT count as compiled Atto guest parity.
use serde_json::{json, Value};
use std::collections::BTreeSet;

const CONTRACT: &str = include_str!("../contracts/v1/core-borrow-checking-parity-v1.json");
const BACKENDS: [&str; 11] = [
    "java", "llvm", "net", "wasm", "rs", "forge", "cranelift",
    "js", "atto", "hdl", "realtime",
];

fn exact_strings(v: &Value, expected: &[&str]) -> bool {
    let Some(items) = v.as_array() else { return false; };
    if items.len() != expected.len() { return false; }
    let set: BTreeSet<&str> = items.iter().filter_map(Value::as_str).collect();
    set.len() == expected.len() && set == expected.iter().copied().collect()
}

fn valid(v: &Value) -> bool {
    if v["schema"] != "atto.core.borrow_checked_libs.v1"
        || v["status"] != "ALL_NOT_EXECUTED"
        || v["implementation"] != "EACH_BACKEND_OWNS_INDEPENDENT_ATTO_CORE_ALLOC_STD"
        || v["abi_promotion"] != "DENY_UNTIL_COMPILED_CHECKER_AND_GUEST"
        || !exact_strings(&v["copy_abi"], &["int", "bool"])
        || !exact_strings(&v["required_positive"], &[
            "copy_scalar_allowed", "move_new_owner_allowed",
        ])
        || !exact_strings(&v["required_negative"], &[
            "use_after_move_denied", "borrowed_self_escape_denied",
            "mut_shared_overlap_denied", "suspension_borrow_crossing_denied",
            "import_noncopy_without_contract_denied", "copy_only_generics_guarded",
        ])
    {
        return false;
    }
    let Some(rows) = v["backends"].as_array() else { return false; };
    if rows.len() != BACKENDS.len() { return false; }
    let mut seen = BTreeSet::new();
    for row in rows {
        let Some(backend) = row["backend"].as_str() else { return false; };
        if !BACKENDS.contains(&backend) || !seen.insert(backend)
            || row["repo"] != format!("atto-lang/atto-source.{backend}")
            || row["stage"] != if backend == "llvm" {
                "SCALAR_COPY_GATE_STAGED_UNVERIFIED"
            } else {
                "UNASSESSED"
            }
            || row["guest_status"] != "NOT_EXECUTED"
            || row["executed_cases"].as_u64() != Some(0)
            || row["receipt"] != "NONE"
            || row.as_object().is_none_or(|obj| obj.len() != 6)
        {
            return false;
        }
    }
    seen.len() == BACKENDS.len() && v.as_object().is_some_and(|obj| obj.len() == 8)
}

#[test]
fn exact_eleven_compiler_rows_require_source_and_guest_borrow_proofs() {
    let data: Value = serde_json::from_str(CONTRACT).expect("JSON contract");
    assert!(valid(&data), "source-only compiler parity contract drifted");
}

#[test]
fn forged_borrow_success_or_removed_ownership_obligation_fails_closed() {
    let initial: Value = serde_json::from_str(CONTRACT).unwrap();
    for backend in BACKENDS {
        let mut data = initial.clone();
        let item = data["backends"].as_array_mut().unwrap().iter_mut()
            .find(|v| v["backend"] == backend).unwrap();
        item["guest_status"] = json!("PASS");
        assert!(!valid(&data), "forged {backend} guest PASS");
    }
    for item in [
        "use_after_move_denied",
        "mut_shared_overlap_denied",
        "suspension_borrow_crossing_denied",
        "copy_only_generics_guarded",
    ] {
        let mut data = initial.clone();
        data["required_negative"].as_array_mut().unwrap().retain(|v| v != item);
        assert!(!valid(&data), "missing ownership requirement: {item}");
    }
    let mut data = initial.clone();
    data["backends"][0]["stage"] = json!("BORROW_PARITY_PASS");
    assert!(!valid(&data), "source state cannot substitute for real execution");
}
