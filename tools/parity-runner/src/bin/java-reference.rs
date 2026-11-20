//! Pinned Java reference CLI adapter. Guest sources are never copied or rewritten.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

#[derive(Deserialize, Serialize)]
struct PinnedFile {
    path: PathBuf,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
struct Bundle {
    compiler_commit: String,
    compiler_tree: String,
    compiler_root: PathBuf,
    java: PathBuf,
    java_version: String,
    maven_version: String,
    mode: String,
    classpath: Vec<PathBuf>,
    native_library: PathBuf,
    files: Vec<PinnedFile>,
}

fn hash(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn checked(command: &mut Command) -> Result<String, String> {
    let output = command.output().map_err(|e| e.to_string())?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success() {
        return Err(format!("command failed ({}): {text}", output.status));
    }
    Ok(text.trim().to_string())
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    checked(Command::new("git").arg("-C").arg(root).args(args))
}

fn verify(bundle: &Bundle) -> Result<(), String> {
    if bundle.mode != "aot" || bundle.files.is_empty() {
        return Err("invalid Java reference bundle".into());
    }
    for file in &bundle.files {
        if hash(&file.path)? != file.sha256 {
            return Err(format!("pinned file changed: {}", file.path.display()));
        }
    }
    // Every path used by the launcher must have a content pin.
    for path in bundle
        .classpath
        .iter()
        .chain([&bundle.java, &bundle.native_library])
    {
        if !bundle.files.iter().any(|file| &file.path == path) {
            return Err(format!("unpinned launcher input: {}", path.display()));
        }
    }
    Ok(())
}

fn prepare(args: &[String]) -> Result<(), String> {
    if args.len() != 4 {
        return Err("prepare requires COMPILER_ROOT EXPECTED_COMMIT JAVA_HOME BUNDLE_JSON".into());
    }
    let root = fs::canonicalize(&args[0]).map_err(|e| e.to_string())?;
    let commit = git(&root, &["rev-parse", "HEAD"])?;
    if args[1].len() != 40 || commit != args[1] {
        return Err(format!(
            "compiler commit mismatch: expected {}, got {commit}",
            args[1]
        ));
    }
    git(&root, &["diff", "--quiet", "HEAD", "--"])?;
    if !git(&root, &["ls-files", "--others", "--exclude-standard"])?.is_empty() {
        return Err("compiler checkout contains untracked inputs".into());
    }
    let java_home = fs::canonicalize(&args[2]).map_err(|e| e.to_string())?;
    let java = java_home.join("bin/java");
    let java_version = checked(Command::new(&java).arg("-version"))?;
    let maven_version = checked(
        Command::new("mvn")
            .env("JAVA_HOME", &java_home)
            .arg("-version"),
    )?;
    // A fresh build is required: do not trust previously generated target/classes.
    let build = checked(
        Command::new("mvn")
            .current_dir(&root)
            .env("JAVA_HOME", &java_home)
            .args([
                "-B",
                "-q",
                "-DskipTests",
                "clean",
                "package",
                "dependency:build-classpath",
                "-Dmdep.includeScope=runtime",
                "-Dmdep.outputFile=target/parity-classpath.txt",
            ]),
    )?;
    eprintln!("{build}");
    git(&root, &["diff", "--quiet", "HEAD", "--"])?;
    let mut classpath = vec![root.join("target/oreslang-source-0.1.0-SNAPSHOT.jar")];
    let dependencies =
        fs::read_to_string(root.join("target/parity-classpath.txt")).map_err(|e| e.to_string())?;
    classpath.extend(env::split_paths(dependencies.trim()));
    let native_library = root.join("target/native").join(format!(
        "{}oresthread{}",
        env::consts::DLL_PREFIX,
        env::consts::DLL_SUFFIX
    ));
    let mut paths = classpath.clone();
    paths.extend([
        java.clone(),
        native_library.clone(),
        java_home.join("release"),
        java_home.join("lib/modules"),
    ]);
    paths.push(java_home.join("lib/server").join(format!(
        "{}jvm{}",
        env::consts::DLL_PREFIX,
        env::consts::DLL_SUFFIX
    )));
    let files = paths
        .into_iter()
        .map(|path| {
            Ok(PinnedFile {
                sha256: hash(&path)?,
                path,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let bundle = Bundle {
        compiler_commit: commit,
        compiler_tree: git(&root, &["rev-parse", "HEAD^{tree}"])?,
        compiler_root: root,
        java,
        java_version,
        maven_version,
        mode: "aot".into(),
        classpath,
        native_library,
        files,
    };
    verify(&bundle)?;
    fs::write(
        &args[3],
        serde_json::to_vec_pretty(&bundle).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn launch(bundle: &Bundle, source: &str, check: bool) -> Result<i32, String> {
    verify(bundle)?;
    let source = fs::canonicalize(source).map_err(|e| e.to_string())?;
    let classpath = env::join_paths(&bundle.classpath).map_err(|e| e.to_string())?;
    let mut command = Command::new(&bundle.java);
    command
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .args([
            "--enable-native-access=ALL-UNNAMED",
            "-Dpolyglot.engine.WarnInterpreterOnly=false",
        ])
        .arg(format!(
            "-Dores.thread.native.path={}",
            bundle.native_library.display()
        ))
        .arg("-cp")
        .arg(classpath)
        .arg("dev.oreslang.launcher.OresMain")
        .arg("--mode=aot");
    if check {
        command.arg("--check");
    }
    let status = command.arg(source).status().map_err(|e| e.to_string())?;
    status
        .code()
        .ok_or_else(|| "Java process terminated by signal".into())
}

fn execute(bundle: &Bundle, bundle_path: &str, output: &str) -> Result<i32, String> {
    verify(bundle)?;
    let adapter = env::current_exe().map_err(|e| e.to_string())?;
    let bundle_path = fs::canonicalize(bundle_path).map_err(|e| e.to_string())?;
    let runner = adapter.with_file_name(format!("ores-parity-runner{}", env::consts::EXE_SUFFIX));
    let template = |mode: &str| {
        format!(
            "{} {mode} {} {{source}}",
            shell_words::quote(&adapter.to_string_lossy()),
            shell_words::quote(&bundle_path.to_string_lossy())
        )
    };
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let corpus_commit = git(&corpus_root, &["rev-parse", "HEAD"])?;
    let corpus_tree = git(&corpus_root, &["rev-parse", "HEAD^{tree}"])?;
    let corpus_diff = git(&corpus_root, &["diff", "HEAD", "--"])?;
    let pending = format!("{output}.pending-{}", std::process::id());
    let status = Command::new(&runner)
        .env("ORES_PARITY_JAVA_COMPILE", template("check"))
        .env("ORES_PARITY_JAVA_RUN", template("run"))
        .args([
            "--execute",
            "--allow-unvalidated",
            "--backend",
            "java",
            "--timeout",
            "30",
            "--output",
            &pending,
        ])
        .status()
        .map_err(|e| e.to_string())?;
    verify(bundle)?;
    if git(&corpus_root, &["rev-parse", "HEAD"])? != corpus_commit
        || git(&corpus_root, &["diff", "HEAD", "--"])? != corpus_diff
    {
        return Err("corpus changed during execution; report is not admitted".into());
    }
    fs::rename(&pending, output)
        .map_err(|e| format!("runner did not publish a fresh report: {e}"))?;
    let receipt = serde_json::json!({
        "schema_version": 1, "scope": "java-reference-exploratory-not-parity-proof",
        "compiler": bundle, "adapter_sha256": hash(&adapter)?, "runner_sha256": hash(&runner)?,
        "corpus_commit": corpus_commit, "corpus_tree": corpus_tree,
        "corpus_tracked_diff_sha256": format!("{:x}", Sha256::digest(corpus_diff.as_bytes())),
        "corpus_tracked_dirty": !corpus_diff.is_empty(),
        "report_sha256": hash(Path::new(output))?, "runner_exit_code": status.code(),
    });
    fs::write(
        format!("{output}.identity.json"),
        serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    status
        .code()
        .ok_or_else(|| "runner terminated by signal".into())
}

fn run() -> Result<i32, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("prepare") {
        prepare(&args[1..])?;
        return Ok(0);
    }
    if args.len() != 3 {
        return Err("usage: java-reference prepare COMPILER_ROOT COMMIT JAVA_HOME BUNDLE_JSON | check|run BUNDLE_JSON SOURCE | execute BUNDLE_JSON REPORT_JSON".into());
    }
    let bundle: Bundle = serde_json::from_slice(&fs::read(&args[1]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    match args[0].as_str() {
        "check" => launch(&bundle, &args[2], true),
        "run" => launch(&bundle, &args[2], false),
        "execute" => execute(&bundle, &args[1], &args[2]),
        _ => Err("unknown Java adapter command".into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("java-reference infrastructure error: {error}");
            ExitCode::from(125)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_detects_changed_artifact() {
        let path = env::temp_dir().join(format!("java-pin-{}", std::process::id()));
        fs::write(&path, b"original").unwrap();
        let pin = PinnedFile {
            sha256: hash(&path).unwrap(),
            path: path.clone(),
        };
        let mut bundle = Bundle {
            compiler_commit: "test".into(),
            compiler_tree: "test".into(),
            compiler_root: path.clone(),
            java: path.clone(),
            java_version: "test".into(),
            maven_version: "test".into(),
            mode: "aot".into(),
            classpath: vec![path.clone()],
            native_library: path.clone(),
            files: vec![pin],
        };
        assert!(verify(&bundle).is_ok());
        bundle.classpath.push(path.with_extension("unpinned"));
        assert!(verify(&bundle)
            .unwrap_err()
            .contains("unpinned launcher input"));
        bundle.classpath.pop();
        fs::write(&path, b"changed").unwrap();
        assert!(verify(&bundle).unwrap_err().contains("pinned file changed"));
        fs::remove_file(path).unwrap();
    }
}
