//! Source-only integrity gate for the pinned 10-provider namespace fixture.
//! This proves the checked-in synthetic fixture, not any backend execution.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_SHA256: &str = "5986c589ab92259065f278ae6771e86065aa4f10776bfd1e55dbe31e7003d394";
const SOURCES_SHA256: &str = "e6e794b0a96d94797f571a8b29ae69f9675bacb6818d9d9862c4213bf79f9d50";

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/package-namespace-v2")
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn namespace_for(i: usize) -> &'static str {
    match i {
        0..=3 => "billing",
        4..=6 => "identity",
        7..=9 => "telemetry",
        _ => panic!("unexpected provider index"),
    }
}

fn source_bytes(i: usize) -> String {
    format!(
        "namespace {};\npub fnc function{}(): int {{ return {}; }}\n",
        namespace_for(i),
        i,
        i
    )
}

fn source_pin(files: &BTreeMap<String, String>) -> String {
    let mut serialized = String::new();
    for (relative, source) in files {
        serialized.push_str(relative);
        serialized.push('\n');
        serialized.push_str(&digest(source.as_bytes()));
        serialized.push('\n');
    }
    digest(serialized.as_bytes())
}

fn sources_from_checkout(root: &Path) -> BTreeMap<String, String> {
    let checkout = root.join("checkout");
    let canonical_checkout = checkout.canonicalize().expect("checkout root exists");
    let mut sources = BTreeMap::new();
    for i in 0..10 {
        let relative = format!("src/unit{}.ores", i);
        let path = checkout.join(&relative);
        let canonical = path.canonicalize().expect("manifest source exists");
        assert!(
            canonical.starts_with(&canonical_checkout),
            "provider escaped package root"
        );
        let contents = fs::read_to_string(path).expect("read checkout provider");
        assert_eq!(
            contents,
            source_bytes(i),
            "unreviewed provider content: {relative}"
        );
        assert!(sources.insert(relative, contents).is_none());
    }
    // Unlisted sources cannot silently become namespace providers. The fixture
    // itself is also limited to the ten audited provider .ores files.
    let actual_count = fs::read_dir(checkout.join("src"))
        .expect("src exists")
        .map(|e| e.expect("directory entry"))
        .count();
    assert_eq!(actual_count, 10, "unexpected unlisted provider in fixture");
    sources
}

#[test]
fn pinned_checkout_and_client_match_exact_manifest_and_source_bytes() {
    let root = fixture_dir();
    let sources = sources_from_checkout(&root);

    let mut expected_checkout = String::from(
        "schema_version = \"2\"\n[project]\nroot = \".\"\n[packages.checkout]\nfiles = [\n",
    );
    for i in 0..10 {
        expected_checkout.push_str(&format!("  \"src/unit{}.ores\",\n", i));
    }
    expected_checkout.push_str("]\n");
    let checkout_manifest =
        fs::read(root.join("checkout/.ores.pkg.toml")).expect("checkout manifest");
    assert_eq!(
        checkout_manifest,
        expected_checkout.as_bytes(),
        "manifest must enumerate only the ten verified sources"
    );
    assert_eq!(digest(&checkout_manifest), MANIFEST_SHA256);
    assert_eq!(source_pin(&sources), SOURCES_SHA256);

    let expected_client = format!(
        "schema_version = \"2\"\n[project]\nroot = \".\"\n[packages.client]\n\
         files = [\"src/main.ores\"]\n[namespace_dependencies.checkout]\n\
         manifest = \"../checkout/.ores.pkg.toml\"\npackage = \"checkout\"\n\
         manifest_sha256 = \"{}\"\nsources_sha256 = \"{}\"\n",
        MANIFEST_SHA256, SOURCES_SHA256
    );
    assert_eq!(
        fs::read_to_string(root.join("client/.ores.pkg.toml")).expect("client manifest"),
        expected_client,
        "client must pin one exact dependency manifest and source snapshot"
    );
    let expected_main = concat!(
        "namespace app;\n",
        "import namespace billing as payments from \"checkout\";\n",
        "pub fnc main(): void {\n",
        "  stdio.println(payments.function0());\n",
        "  stdio.println(payments.function1());\n",
        "  stdio.println(payments.function2());\n",
        "  stdio.println(payments.function3());\n",
        "  return;\n",
        "}\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("client/src/main.ores")).expect("client entry"),
        expected_main
    );
    let spec: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("fixture.json")).expect("fixture metadata"),
    )
    .expect("valid fixture metadata");
    assert_eq!(spec["manifest_sha256"], MANIFEST_SHA256);
    assert_eq!(spec["sources_sha256"], SOURCES_SHA256);
    assert_eq!(spec["provider_count"], 10);
    assert_eq!(spec["namespaces"]["billing"], 4);
    assert_eq!(spec["namespaces"]["identity"], 3);
    assert_eq!(spec["namespaces"]["telemetry"], 3);
}

#[test]
fn source_tampering_and_package_membership_changes_break_pins() {
    let root = fixture_dir();
    let mut sources = sources_from_checkout(&root);
    assert_eq!(source_pin(&sources), SOURCES_SHA256);
    let tampered = sources
        .get_mut("src/unit3.ores")
        .expect("provider is present");
    tampered.push_str("// modified without pin update\n");
    assert_ne!(source_pin(&sources), SOURCES_SHA256);

    let original = fs::read(root.join("checkout/.ores.pkg.toml")).expect("manifest");
    assert_eq!(digest(&original), MANIFEST_SHA256);
    let mut added_member = original;
    added_member.extend_from_slice(b"# unapproved extra member\n");
    assert_ne!(digest(&added_member), MANIFEST_SHA256);
}
