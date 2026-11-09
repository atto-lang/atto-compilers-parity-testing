//! Structural spec-to-parity traceability gate. This is NOT compiler or guest evidence.
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn json(path: &str) -> Value {
    serde_json::from_slice(&fs::read(root().join(path)).expect("read JSON")).expect("valid JSON")
}

fn strings(v: &Value) -> Vec<&str> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|s| s.as_str().expect("string"))
        .collect()
}

#[test]
fn spec_alignment_does_not_conflate_backends_or_claim_execution() {
    let a = json("contracts/spec-alignment-v1.json");
    let manifest = json("manifest.json");
    assert_eq!(a["schema_version"], 1);
    assert_eq!(a["contract_id"], "atto.compiler-parity.spec-bridge.v1");
    assert_eq!(a["status"], "draft_traceability_only");
    assert_eq!(a["specification"]["edition"], "0.1");
    assert_eq!(
        a["specification"]["repository"],
        "ores-truffle-oreslang/oreslang-language-specs.md"
    );
    let spec_commit = a["specification"]["reviewed_commit"]
        .as_str()
        .expect("immutable spec commit");
    assert_eq!(spec_commit.len(), 40);
    assert!(spec_commit.bytes().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(a["source_policy"]["canonical_new_extension"], ".atto");
    assert_eq!(a["source_policy"]["existing_legacy_extension"], ".ores");
    assert_eq!(a["source_policy"]["rewrite_legacy_in_place"], false);
    assert_eq!(a["source_policy"]["identical_source_per_backend"], true);
    assert_eq!(a["claim_policy"]["registry_is_execution_evidence"], false);
    assert_eq!(a["claim_policy"]["registry_is_spec_conformance"], false);
    assert_eq!(
        a["claim_policy"]["new_cases_require_independent_oracle_review"],
        true
    );

    let required = strings(&a["targets"]["required"]);
    assert_eq!(
        required,
        vec!["llvm", "java", "wasm", "dotnet", "forge", "cranelift"]
    );
    assert_eq!(strings(&a["targets"]["optional"]), vec!["js"]);
    assert_eq!(
        a["targets"]["required"],
        manifest["policy"]["target_backends"]
    );
    assert_eq!(
        a["targets"]["optional"],
        manifest["policy"]["optional_backends"]
    );
    assert_eq!(manifest["policy"]["required_parity"], true);
    assert_eq!(manifest["policy"]["unsupported_is_pass"], false);
    assert_eq!(manifest["policy"]["infrastructure_failure_is_pass"], false);
    assert_eq!(
        manifest["policy"]["negative_tests_require_diagnostic_classification"],
        true
    );
}

#[test]
fn links_reference_existing_cases_without_promotion_to_parity() {
    let bridge = json("contracts/spec-alignment-v1.json");
    let manifest = json("manifest.json");
    let checks = json("checks/manifest.json");

    let legacy: BTreeSet<&str> = manifest["cases"]
        .as_array()
        .expect("legacy cases")
        .iter()
        .map(|c| c["id"].as_str().expect("legacy ID"))
        .collect();
    let named: BTreeSet<&str> = checks["checks"]
        .as_array()
        .expect("multi-file checks")
        .iter()
        .map(|c| c["name"].as_str().expect("check name"))
        .collect();
    let spec_catalogue: BTreeSet<&str> = [
        "parse.define_pub_module",
        "parse.match_forms",
        "actor.sealed_state_access",
        "actor.mailbox_backpressure",
        "actor.critical_write_exclusive",
        "ownership.double_mut_borrow",
        "channel.select_cancel",
        "types.match_return_inference",
    ]
    .into_iter()
    .collect();

    let mut unique = BTreeSet::new();
    let links = bridge["links"].as_array().expect("links");
    assert!(!links.is_empty(), "traceability registry must not be empty");
    for link in links {
        let spec_id = link["spec_fixture_id"].as_str().expect("spec fixture ID");
        let suite = link["corpus"]["suite"].as_str().expect("suite");
        let id = link["corpus"]["id"].as_str().expect("case ID");
        assert!(
            spec_catalogue.contains(spec_id),
            "unknown spec ID: {spec_id}"
        );
        assert_eq!(link["relation"], "related_feature_only");
        assert!(
            link["review_note"].as_str().is_some_and(|s| s.len() >= 20),
            "every link needs an explicit coverage caveat"
        );
        assert!(
            unique.insert((spec_id, suite, id)),
            "duplicate registry link"
        );
        match suite {
            "legacy" => assert!(legacy.contains(id), "unknown legacy case: {id}"),
            "checks" => assert!(named.contains(id), "unknown named check: {id}"),
            _ => panic!("unknown corpus suite: {suite}"),
        }
    }
}
