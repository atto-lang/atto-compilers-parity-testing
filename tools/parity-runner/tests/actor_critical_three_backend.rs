//! Source identity and staged negative-case contract for all three native Atto
//! compilers. These tests do NOT compile or execute the Atto guest.
use std::{collections::BTreeSet, fs, path::PathBuf, process::Command};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn contract() -> serde_json::Value {
    let data = fs::read(root().join("contracts/atto-actor-critical-v1/contract.json"))
        .expect("read actor critical contract");
    serde_json::from_slice(&data).expect("valid strict JSON")
}

const SNAPSHOT: &str = "      // Critical-read yields a detached Copy scalar. The loan ends before
      // mailbox publication; actor state and its borrow never escape.
      // Compiler support for this expression is still an execution gate.
      val int snapshot = self.handle_state(critical-read |s| -> s.counter);
      self.send(snapshot);";

const BAD_SEND: &str = "      // Intentional compiler-negative: sending while a critical-read
      // state loan is active violates actor-state ownership.
      self.handle_state(critical-read |s| -> {
        self.send(s.counter);
      });";

const BAD_WRITE: &str = "      // Intentional compiler-negative: a critical-read view cannot mutate
      // its actor-owned state, even when this value is a scalar.
      self.handle_state(critical-read |s| -> {
        s.counter = s.counter + 1;
      });
      self.send(1);";

#[test]
fn actor_critical_contract_is_staged_not_false_certified() {
    let c = contract();
    assert_eq!(c["schema_version"], 1);
    assert_eq!(c["contract_id"], "atto-actor-critical-v1");
    assert_eq!(c["status"], "AUTHORED_NOT_EXECUTED");
    assert_eq!(c["source_policy"]["canonical_extension"], ".atto");
    assert_eq!(c["source_policy"]["guest_first"], true);
    assert_eq!(c["source_policy"]["host_substitution_denied"], true);

    let required = c["required_backends"].as_array().unwrap();
    assert_eq!(required.len(), 3);
    let actual: BTreeSet<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(actual, BTreeSet::from(["java", "llvm", "dotnet"]));
    for backend in &actual {
        assert_eq!(c["backend_results"][backend], "NOT_EXECUTED");
    }
    assert_eq!(c["performance_gate"]["status"], "NOT_BENCHMARKED");
    assert_eq!(c["performance_gate"]["minimum_median_ratio"], 0.70);
    assert_eq!(c["performance_gate"]["not_proved_by_fixture"], true);
    let required_proofs = c["admission"]["requires"].as_array().unwrap();
    assert!(required_proofs.len() >= 10);
    for gate in ["actual_parser_type_and_borrow_checker", "executed_guest_output",
                 "targeted_semantic_diagnostic_not_generic_parse_error",
                 "nonzero_runner_and_steps", "no_backend_substitution"] {
        assert!(required_proofs.iter().any(|item| item == gate), "missing {gate}");
    }
}

#[test]
fn positive_fixture_is_the_exact_unchanged_four_broker_git_blob() {
    let c = contract();
    let relative = c["positive"]["path"].as_str().unwrap();
    let path = root().join(relative);
    assert!(relative.ends_with(".atto") && !relative.contains(".."));
    let source = fs::read_to_string(&path).expect("positive actor fixture");
    assert!(source.starts_with("use namespace guest;\n"));
    assert_eq!(source.matches(SNAPSHOT).count(), 1);
    assert!(!source.contains("self.send(s.counter)"));
    assert!(source.contains("define actor ArenaIsolationProbe as"));
    assert!(source.contains("self.handle_state(critical-write |s| -> {"));
    assert!(source.contains("ATTO_TWO_ACTOR_LOGICAL_ISOLATION=PASS"));

    // A Git blob identity includes the blob header and original bytes. Using
    // 'git hash-object' avoids mistaking a source filename for an exact match.
    let output = Command::new("git")
        .arg("hash-object")
        .arg(&path)
        .output()
        .expect("Git is required to authenticate the source blob");
    assert!(output.status.success(), "Git source hash failed");
    let observed = String::from_utf8(output.stdout).unwrap();
    assert_eq!(observed.trim(), c["positive"]["git_blob_sha"].as_str().unwrap());
    assert_eq!(c["positive"]["status"], "AUTHORED_NOT_EXECUTED");
}

#[test]
fn negatives_are_only_targeted_mutations_of_the_pinned_positive_source() {
    let c = contract();
    let base_path = c["positive"]["path"].as_str().unwrap();
    let original = fs::read_to_string(root().join(base_path)).unwrap();
    let negatives = c["negatives"].as_array().unwrap();
    assert_eq!(negatives.len(), 2);
    let mut seen = BTreeSet::new();
    for entry in negatives {
        let id = entry["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(entry["status"], "AUTHORED_NOT_EXECUTED");
        let path = entry["path"].as_str().unwrap();
        assert!(path.starts_with("contracts/atto-actor-critical-v1/fixtures/"));
        assert!(path.ends_with(".atto") && !path.contains(".."));
        let negative = fs::read_to_string(root().join(path)).unwrap();
        let (mutation, category) = match id {
            "deny-send-inside-read" => (BAD_SEND, "actor_state_loan_escape_to_mailbox"),
            "deny-write-inside-read" => (BAD_WRITE, "write_through_read_only_actor_loan"),
            _ => panic!("unexpected negative case: {id}"),
        };
        assert_eq!(entry["expected_category"], category);
        assert_eq!(negative, original.replace(SNAPSHOT, mutation));
    }
}

#[test]
fn four_original_broker_pr_heads_are_declared_but_not_treated_as_execution() {
    let c = contract();
    let mirrors = c["mirrors"].as_array().unwrap();
    assert_eq!(mirrors.len(), 4);
    let expected = BTreeSet::from([
        "atto-poc/atto-nats-clone.atto",
        "atto-poc/atto-kafka-clone.atto",
        "atto-poc/atto-rmq-clone.atto",
        "ores-truffle-oreslang/atto-redis-clone.atto",
    ]);
    let mut observed = BTreeSet::new();
    for item in mirrors {
        let repo = item["repo"].as_str().unwrap();
        assert!(observed.insert(repo));
        assert!(item["pr"].as_u64().unwrap() > 0);
        let sha = item["commit"].as_str().unwrap();
        assert!(sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
    }
    assert_eq!(observed, expected);
    // These identities must be rechecked against GitHub at execution time.
    // Merely declaring a head SHA does NOT authenticate a workflow receipt.
}
