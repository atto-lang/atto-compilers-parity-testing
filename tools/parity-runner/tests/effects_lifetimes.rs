use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn git_blob(bytes: &[u8]) -> String {
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("git hash-object available");
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().expect("hash-object execution");
    assert!(output.status.success(), "Git blob hash failed");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[test]
fn effects_and_lifetimes_are_exact_blob_candidates_not_guest_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let folder = root.join("contracts/attolang-effects-lifetimes-v1");
    let contract_bytes = fs::read(folder.join("contract.json")).expect("contract");
    let data: serde_json::Value = serde_json::from_slice(&contract_bytes).unwrap();
    let hashes: serde_json::Value =
        serde_json::from_slice(&fs::read(folder.join("blob-manifest.json")).unwrap()).unwrap();
    assert_eq!(data["schema_version"], 1);
    assert_eq!(data["contract_id"], "attolang-effects-lifetimes-v1");
    assert_eq!(data["status"], "TARGET_NOT_EXECUTED");
    assert_eq!(hashes["status"], "SOURCE_ONLY_NOT_EXECUTED");
    assert_eq!(hashes["contract_blob_sha"], git_blob(&contract_bytes));
    assert_eq!(data["canonical_extension"], ".atto");
    assert_eq!(data["evidence"]["baseline"], "NOT_EXECUTED");
    assert_eq!(data["evidence"]["eligible_tier"], "ORES_GUEST_EXECUTED");
    assert_eq!(
        data["required_backends"],
        serde_json::json!(["llvm", "java", "wasm", "dotnet", "forge", "cranelift"])
    );
    let cases = data["cases"].as_array().unwrap();
    let mirrors = hashes["fixtures"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    assert_eq!(mirrors.len(), cases.len());
    let mut names = HashSet::new();
    for (case, mirror) in cases.iter().zip(mirrors.iter()) {
        let id = case["id"].as_str().unwrap();
        assert!(names.insert(id));
        assert_eq!(mirror["id"], id);
        assert_eq!(case["status"], "AUTHORED_NOT_EXECUTED");
        let path = case["path"].as_str().unwrap();
        assert_eq!(path, format!("contracts/attolang-effects-lifetimes-v1/fixtures/{id}.atto"));
        assert_eq!(mirror["path"], path);
        let bytes = fs::read(root.join(path)).expect("fixture");
        assert_eq!(git_blob(&bytes), mirror["git_blob_sha"].as_str().unwrap());
        assert!(bytes.ends_with(b"\n"));
        assert!(bytes.windows(b"rt borrow owner".len()).any(|part| part == b"rt borrow owner"));
        match case["kind"].as_str().unwrap() {
            "compile_and_execute" => {
                assert_eq!(case["expected_exit"], 42);
                assert!(case["diagnostic"].is_null());
            }
            "compile_reject" => {
                assert!(case["expected_exit"].is_null());
                assert!(matches!(
                    case["diagnostic"].as_str().unwrap(),
                    "move_while_borrowed" | "incompatible_borrow" | "borrow_across_suspension"
                ));
            }
            other => panic!("unexpected candidate kind: {other}"),
        }
    }
}

#[test]
fn one_byte_mutation_breaks_git_blob_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = fs::read(root.join(
        "contracts/attolang-effects-lifetimes-v1/fixtures/move-while-borrowed-reject.atto"
    )).unwrap();
    let mut mutated = source.clone();
    mutated.push(b'!');
    assert_ne!(git_blob(&source), git_blob(&mutated));
}
