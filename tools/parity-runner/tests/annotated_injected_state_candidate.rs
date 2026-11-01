//! Candidate contract integrity only; does not execute any Oreslang compiler.
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

#[test]
fn annotated_state_candidate_is_fail_closed_and_complete() {
    let root = repository_root();
    let path = root.join("candidates/injected-state/manifest.json");
    let manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(manifest["status"], "draft_not_executed");
    assert_eq!(manifest["required_parity"], false);
    assert_eq!(manifest["unsupported_is_pass"], false);
    assert_eq!(manifest["infrastructure_failure_is_pass"], false);
    assert_eq!(manifest["parser_only_reject_is_semantic_pass"], false);

    let backends: Vec<_> = manifest["target_backends"].as_array().unwrap().iter()
        .map(|v| v.as_str().unwrap()).collect();
    assert_eq!(backends, ["java", "llvm", "wasm", "cranelift", "dotnet"]);

    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 7);
    let mut names = BTreeSet::new();
    let mut positive = 0;
    let mut negative = 0;
    let candidate_root = root.join("candidates/injected-state").canonicalize().unwrap();
    for case in cases {
        let name = case["name"].as_str().unwrap();
        assert!(names.insert(name), "duplicate candidate: {name}");
        assert_eq!(case["validation_status"], "candidate_unverified");
        let rel = case["path"].as_str().unwrap();
        assert!(rel.starts_with("candidates/injected-state/"));
        let file = root.join(rel).canonicalize().unwrap();
        assert!(file.starts_with(&candidate_root), "escaped candidate path: {rel}");
        assert_eq!(file.extension().unwrap(), "ores");
        let source = fs::read_to_string(&file).unwrap();
        assert!(!source.trim().is_empty());
        assert!(source.contains("@Inject(MutableStateContainer())"),
            "candidate does not exercise injected annotation: {name}");
        match case["expected_kind"].as_str().unwrap() {
            "run" => {
                positive += 1;
                assert_eq!(case["expected_stdout"], "2");
                assert_eq!(case["expected_exit_code"], 0);
            }
            "compile_reject" => {
                negative += 1;
                assert!(case["expected_diagnostic_category"].as_str().is_some_and(|s| !s.is_empty()));
                let phase = case["required_phase"].as_str().unwrap();
                assert!(["ownership", "type_or_ownership"].contains(&phase));
            }
            kind => panic!("unsupported oracle type: {kind}"),
        }
    }
    assert_eq!((positive, negative), (1, 6));

    // Never silently promote a candidate into the required shared matrix.
    let required = fs::read_to_string(root.join("manifest.json")).unwrap();
    assert!(!required.contains("candidates/injected-state/"));
}
