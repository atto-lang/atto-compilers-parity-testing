//! Host-only GLM/MoE manifest and oracle regression checks.
//! These checks do NOT compile Oreslang or prove four-backend parity.
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const GLM_NAMES: [&str; 20] = [
    "glm-top1-highest",
    "glm-top1-stable-tie",
    "glm-negative-score-ranking",
    "glm-first-expert-capacity",
    "glm-second-expert-capacity",
    "glm-preferred-full-no-fallback",
    "glm-balanced-loads",
    "glm-unbalanced-load-reject",
    "glm-negative-load-reject",
    "glm-assignment-total",
    "glm-top-two-load-count",
    "glm-invalid-topk",
    "glm-fresh-generation",
    "glm-stale-gradient-reject",
    "glm-cancellation-reject",
    "glm-duplicate-step-reject",
    "glm-skipped-step-reject",
    "glm-fixed-gate-prediction",
    "glm-fixed-gate-weight-gradient",
    "glm-fixed-gate-sgd-loss",
];

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn manifest() -> Value {
    let bytes = fs::read(root().join("manifest.json")).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn exactly_twenty_glm_programs_are_required_on_all_backends() {
    let manifest = manifest();
    let rows = manifest["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 51);
    let backends = manifest["policy"]["target_backends"].as_array().unwrap();
    for backend in ["java", "llvm", "wasm", "dotnet"] {
        assert!(backends.iter().any(|item| item.as_str() == Some(backend)));
    }

    let mut all_ids = BTreeSet::new();
    let mut all_paths = BTreeSet::new();
    let mut actual_glm_names = BTreeSet::new();
    let mut n_glm = 0usize;
    for row in rows {
        let id = row["id"].as_str().unwrap();
        let path = row["path"].as_str().unwrap();
        assert!(all_ids.insert(id.to_string()), "duplicate case ID: {id}");
        assert!(all_paths.insert(path.to_string()), "duplicate source path: {path}");
        if row["group"] != "glm-moe" {
            continue;
        }
        n_glm += 1;
        let number: usize = id.parse().unwrap();
        assert!((32..=51).contains(&number), "unknown GLM case: {id}");
        let name = row["name"].as_str().unwrap();
        assert_eq!(name, GLM_NAMES[number - 32]);
        assert!(actual_glm_names.insert(name.to_string()));
        assert_eq!(path, format!("programs/{number}-{name}.ores"));
        assert_eq!(row["expected_kind"], "exit");
        assert_eq!(row["expected_value"], 42);
        assert_eq!(row["expected_diagnostic_category"], Value::Null);
        assert_eq!(row["validation_status"], "candidate");
        assert_eq!(row["required_on_all_backends"], true);
        let bytes = fs::read(root().join(path)).unwrap();
        assert!((80..=8192).contains(&bytes.len()), "invalid source budget {path}");
        let source = std::str::from_utf8(&bytes).unwrap();
        assert!(source.contains("pub fnc main(): int"), "missing entry: {path}");
        assert!(source.contains("return 42;"), "missing success oracle: {path}");
        assert!(source.contains("return 1;"), "missing failure oracle: {path}");
        assert!(!source.contains("import "), "no backend-local imports: {path}");
        assert!(!source.contains("std::") && !source.contains("System."),
            "backend-specific code in {path}");
        if number >= 49 {
            assert!(row["features"].as_array().unwrap().iter()
                .any(|v| v.as_str() == Some("float")));
            assert!(source.contains("float "), "float frontier must be exercised");
        } else {
            assert!(!source.contains("float "), "integer control case became float");
        }
    }
    assert_eq!(n_glm, 20);
    assert_eq!(actual_glm_names.len(), 20);
}

fn route_top1(a: i64, b: i64) -> i64 {
    if a >= b { 0 } else { 1 }
}

fn capacity_route(a: i64, b: i64, l0: i64, l1: i64, cap: i64) -> i64 {
    if cap <= 0 || l0 < 0 || l1 < 0 || l0 > cap || l1 > cap {
        return -1;
    }
    let winner = route_top1(a, b);
    if (if winner == 0 { l0 } else { l1 }) >= cap {
        -1
    } else {
        winner
    }
}

#[test]
fn deterministic_ties_and_no_fallback_oracle() {
    assert_eq!(route_top1(9, 2), 0);
    assert_eq!(route_top1(3, 3), 0);
    assert_eq!(route_top1(-5, -1), 1);
    assert_eq!(capacity_route(7, 6, 2, 0, 2), -1);
    assert_eq!(capacity_route(-1, 0, 2, 1, 2), 1);
    assert_eq!(capacity_route(1, 1, 1, 2, 2), 0);
    assert_eq!(capacity_route(1, 1, 2, 0, 2), -1);
}

fn counted_loads(l0: i64, l1: i64, tokens: i64, k: i64, cap: i64) -> bool {
    if tokens <= 0 || k <= 0 || k > 2 || cap <= 0
        || l0 < 0 || l1 < 0 || l0 > cap || l1 > cap
    {
        return false;
    }
    let Some(total) = tokens.checked_mul(k) else { return false; };
    l0.checked_add(l1) == Some(total)
}

#[test]
fn exact_load_count_and_i64_overflow_oracle() {
    assert!(counted_loads(2, 2, 4, 1, 2));
    assert!(!counted_loads(4, 0, 4, 1, 2));
    assert!(!counted_loads(-1, 2, 1, 1, 2));
    assert!(!counted_loads(1, 1, 4, 1, 2));
    assert!(counted_loads(2, 2, 2, 2, 2));
    assert!(!counted_loads(i64::MAX, 1, i64::MAX, 2, i64::MAX));
    // Counting assignments alone does not prove that top-k uses unique experts.
    assert!(counted_loads(2, 0, 1, 2, 2));
}

#[test]
fn gradient_and_sgd_loss_oracles_are_exact_binary_fractions() {
    let gate: f64 = 0.25;
    let predicted = gate * (2.0 * 1.0) + (1.0 - gate) * (2.0 * 3.0);
    assert_eq!(predicted, 5.0);
    let gate = 0.75;
    let gradient_weight0 = 2.0 * (0.0 - 1.0) * gate * 1.0;
    assert_eq!(gradient_weight0, -1.5);
    let old_loss: f64 = 1.0;
    let prediction = 0.75 * 0.375 + 0.25 * 0.125;
    let new_loss = (prediction - 1.0) * (prediction - 1.0);
    assert_eq!(new_loss, 0.47265625);
    assert!(new_loss < old_loss);
}

#[test]
fn stale_and_skipped_step_controls_are_not_validated_as_progress() {
    let active = 7i64;
    let model = 7i64;
    let gradient = 6i64;
    assert!(model == active && gradient != active);
    let prior = 3i64;
    for proposed in [3, 5, -1] {
        assert_ne!(proposed, prior + 1);
    }
    assert_eq!(prior + 1, 4);
}
