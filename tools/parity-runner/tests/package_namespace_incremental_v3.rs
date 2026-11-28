//! Fixture integrity and mutation oracle only, never compiler execution.
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fs,path::{Path,PathBuf}};
fn hash(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn root()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/package-namespace-v3")}
fn pin(files:&BTreeMap<String,String>)->String{
    let mut s=String::new();for (p,b) in files{s.push_str(p);s.push('\n');s.push_str(&hash(b.as_bytes()));s.push('\n');}hash(s.as_bytes())
}
fn read(root:&Path)->(BTreeMap<String,String>,serde_json::Value){
    let spec=serde_json::from_slice(&fs::read(root.join("fixture.json")).unwrap()).unwrap();
    let mut files=BTreeMap::new();
    for e in fs::read_dir(root.join("checkout/src")).unwrap(){
        let e=e.unwrap();assert!(e.file_type().unwrap().is_file());let p=format!("src/{}",e.file_name().to_str().unwrap());
        assert!(files.insert(p,fs::read_to_string(e.path()).unwrap()).is_none());
    }(files,spec)
}
#[test]fn exact_inventory_and_both_pins(){
    let r=root();let (files,spec)=read(&r);assert_eq!(files.len(),12);
    let mut manifest=String::from("schema_version = \"2\"\n[project]\nroot = \".\"\n[packages.checkout]\nfiles = [\n");
    for p in files.keys(){manifest.push_str(&format!("  \"{p}\",\n"));}manifest.push_str("]\n");
    assert_eq!(fs::read_to_string(r.join("checkout/.ores.pkg.toml")).unwrap(),manifest);
    assert_eq!(hash(manifest.as_bytes()),spec["manifest_sha256"].as_str().unwrap());assert_eq!(pin(&files),spec["sources_sha256"].as_str().unwrap());
    for i in 0..10{let ns=if i<4{"billing"}else if i<7{"identity"}else{"telemetry"};
        assert_eq!(files[&format!("src/unit{i}.ores")],format!("namespace {ns};\npub fnc function{i}(): int {{ return {i}; }}\n"));}
    for i in 0..2{assert_eq!(files[&format!("src/local{i}.ores")],format!("pub fnc local{i}(): int {{ return {}; }}\n",10+i));}
    for (client,ns,ids) in [("client","billing",0..4),("observer","identity",4..7)]{
        let m=fs::read_to_string(r.join(client).join(".ores.pkg.toml")).unwrap();
        let expected=format!("schema_version = \"2\"\n[project]\nroot = \".\"\n[packages.{client}]\nfiles = [\"src/main.ores\"]\n[namespace_dependencies.checkout]\nmanifest = \"../checkout/.ores.pkg.toml\"\npackage = \"checkout\"\nmanifest_sha256 = \"{}\"\nsources_sha256 = \"{}\"\n",spec["manifest_sha256"].as_str().unwrap(),spec["sources_sha256"].as_str().unwrap());
        assert_eq!(m,expected);
        let mut source=format!("namespace app;\nimport namespace {ns} as provider from \"checkout\";\npub fnc main(): void {{\n");
        for i in ids{source.push_str(&format!("  stdio.println(provider.function{i}());\n"));}source.push_str("  return;\n}\n");
        assert_eq!(fs::read_to_string(r.join(client).join("src/main.ores")).unwrap(),source);
    }
}
#[test]fn mutation_pins_and_failed_edit_oracle(){
    let (mut files,spec)=read(&root());let initial=pin(&files);
    let original=files["src/unit0.ores"].clone();files.insert("src/unit0.ores".into(),original.replace("return 0","return 42"));
    let edited=pin(&files);assert_ne!(edited,initial);assert_eq!(edited,spec["lanes"]["body_edit"]["repinned_sources_sha256"].as_str().unwrap());
    files.insert("src/unit0.ores".into(),spec["lanes"]["abi_edit"]["new_source"].as_str().unwrap().into());
    assert_eq!(pin(&files),spec["lanes"]["abi_edit"]["repinned_sources_sha256"].as_str().unwrap());
    assert_ne!(pin(&files),edited);files.remove("src/unit0.ores");assert_ne!(pin(&files),initial);
    files.insert("src/injected.ores".into(),original);assert_ne!(pin(&files),initial);
    println!("INCREMENTAL_V3_FIXTURE|12-providers|2-consumers|pins|mutation-oracle-only");
}
