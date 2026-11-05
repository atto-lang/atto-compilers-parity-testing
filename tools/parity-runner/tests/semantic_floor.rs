use std::{collections::HashSet, fs, path::Path};

#[test]
fn semantic_floor_is_staged_and_matches_required_six_backends() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let data = fs::read_to_string(root.join("contracts/attolang-semantic-floor-v1/contract.json"))
        .expect("semantic floor contract");
    let contract: serde_json::Value = serde_json::from_str(&data).expect("valid JSON");
    assert_eq!(contract["schema_version"], 1);
    assert_eq!(contract["contract_id"], "attolang-semantic-floor-v1");
    assert_eq!(contract["status"], "DRAFT_TARGET_NOT_VERIFIED");
    assert_eq!(contract["source_policy"]["canonical_extension"], ".atto");
    assert_eq!(contract["source_policy"]["legacy_admission"], "versioned_migration_only");
    assert_eq!(contract["existing_corpus_policy"], "additive_candidate_lane_not_in_legacy_53_case_pass_matrix");
    assert_eq!(contract["evidence_policy"]["baseline"], "NOT_EXECUTED");
    assert_eq!(contract["evidence_policy"]["eligible_parity_tier"], "ORES_GUEST_EXECUTED");
    let corpus: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("manifest.json")).expect("legacy corpus manifest"),
    ).expect("valid legacy manifest");
    assert_eq!(contract["required_backends"], corpus["policy"]["target_backends"]);
    let backends = contract["required_backends"].as_array().unwrap();
    assert_eq!(backends.len(), 6);
    for backend in backends {
        let id = backend.as_str().unwrap();
        assert!(contract["backend_repositories"][id].as_str().unwrap().starts_with("ores-truffle-oreslang/oreslang-source."));
    }
    let features = contract["features"].as_array().unwrap();
    assert_eq!(features.len(), 9);
    let mut feature_ids = HashSet::new();
    for feature in features {
        assert_eq!(feature["status"], "TARGET_NOT_VERIFIED");
        assert!(feature_ids.insert(feature["id"].as_str().unwrap()));
        assert!(feature["decision"].as_str().unwrap().len() > 30);
    }
    assert!(feature_ids.contains("canonical-core-std-identity"));
    assert!(feature_ids.contains("critical-suspends-effect-separation"));
    let fixtures = contract["fixtures"].as_array().unwrap();
    assert_eq!(fixtures.len(), 4);
    let mut fixture_ids = HashSet::new();
    for fixture in fixtures {
        assert_eq!(fixture["status"], "AUTHORED_NOT_EXECUTED");
        assert!(feature_ids.contains(fixture["feature"].as_str().unwrap()));
        assert!(fixture_ids.insert(fixture["id"].as_str().unwrap()));
        let rel = fixture["path"].as_str().unwrap();
        assert!(rel.starts_with("contracts/attolang-semantic-floor-v1/fixtures/"));
        assert!(rel.ends_with(".atto") && !rel.contains("..") && !rel.contains('\\'));
        let source = fs::read_to_string(root.join(rel)).expect("canonical source exists");
        assert!(source.contains("fnc") && source.ends_with('\n'));
        match fixture["kind"].as_str().unwrap() {
            "compile_and_execute" => {
                assert_eq!(fixture["expected_exit"], 42);
                assert!(fixture["expected_diagnostic_category"].is_null());
            }
            "compile_reject" => {
                assert!(fixture["expected_exit"].is_null());
                assert!(!fixture["expected_diagnostic_category"].as_str().unwrap().is_empty());
            }
            kind => panic!("invalid candidate kind: {kind}"),
        }
    }
}
