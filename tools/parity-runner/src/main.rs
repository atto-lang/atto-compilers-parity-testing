use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// Six independent compiler backends required; JS transpiler optional.
const BACKENDS: [(&str, &str); 7] = [
    ("llvm", "LLVM"),
    ("java", "JAVA"),
    ("wasm", "WASM"),
    ("dotnet", "DOTNET"),
    ("forge", "FORGE"),
    ("cranelift", "CRANELIFT"),
    ("js", "JS"),
];

#[derive(Debug, Deserialize)]
struct LegacyManifest {
    policy: LegacyPolicy,
    cases: Vec<LegacyCase>,
}

#[derive(Debug, Deserialize)]
struct LegacyPolicy {
    target_backends: Vec<String>,
    optional_backends: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyCase {
    id: String,
    name: String,
    path: String,
    expected_kind: String,
    expected_value: Option<i32>,
    expected_diagnostic_category: Option<String>,
    validation_status: String,
    features: Vec<String>,
    required_on_all_backends: bool,
}

#[derive(Debug, Deserialize)]
struct ChecksManifest {
    checks: Vec<CheckRef>,
}

#[derive(Debug, Deserialize)]
struct CheckRef {
    name: String,
    expectation: String,
}

#[derive(Debug, Deserialize)]
struct Expectation {
    name: String,
    entry: String,
    stable_stdout_prefix: String,
    imported_files: usize,
    maturity: String,
    #[serde(default)]
    features: Vec<String>,
    verified: bool,
    #[serde(default)]
    negative_cases: Vec<NegativeExpectation>,
}

#[derive(Debug, Deserialize)]
struct NegativeExpectation {
    path: String,
    expected_diagnostic_category: Option<String>,
    #[allow(dead_code)]
    reason: Option<String>,
}

#[derive(Debug)]
struct TestCase {
    suite: String,
    case_id: String,
    name: String,
    source: PathBuf,
    source_sha256: String,
    graph_sha256: Option<String>,
    graph_files: Option<Vec<String>>,
    expected_kind: String,
    expected_value: Option<i32>,
    expected_diagnostic_category: Option<String>,
    expected_stdout_prefix: Option<String>,
    validation_status: String,
    features: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Invocation {
    status: String,
    exit_code: Option<i32>,
    stdout_hex: String,
    stderr_hex: String,
    duration_ms: u128,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct Record {
    suite: String,
    case_id: String,
    name: String,
    backend: String,
    source_sha256: String,
    graph_sha256: Option<String>,
    graph_files: Option<Vec<String>>,
    validation_status: String,
    features: Vec<String>,
    expected_diagnostic_category: Option<String>,
    actual_diagnostic_category: Option<String>,
    status: String,
    compile: Option<Invocation>,
    run: Option<Invocation>,
}

#[derive(Debug, Serialize)]
struct Corpus {
    legacy_cases: usize,
    multifile_checks: usize,
    negative_probes: usize,
    total_cases: usize,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: u32,
    executed: bool,
    corpus: Corpus,
    cases: usize,
    backends: Vec<String>,
    records: Vec<Record>,
    summary: BTreeMap<String, usize>,
}

fn hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(H[(b >> 4) as usize] as char);
        out.push(H[(b & 0x0f) as usize] as char);
    }
    out
}

fn ascii_lower_hex(input: &str) -> Option<String> {
    if input.len() % 2 != 0 {
        return None;
    }
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let nibble = |value: u8| -> Option<u8> {
        match value {
            b'0'..=b'9' => Some(value - b'0'),
            b'a'..=b'f' => Some(value - b'a' + 10),
            b'A'..=b'F' => Some(value - b'A' + 10),
            _ => None,
        }
    };
    for pair in bytes.chunks_exact(2) {
        let high = nibble(pair[0])?;
        let low = nibble(pair[1])?;
        decoded.push((high << 4 | low).to_ascii_lowercase());
    }
    Some(hex(&decoded))
}

type DiagnosticRules = BTreeMap<String, BTreeMap<String, Vec<String>>>;

fn classify_diagnostic(
    rules: &DiagnosticRules,
    backend: &str,
    invocation: &Invocation,
) -> Option<String> {
    let backend_rules = rules.get(backend)?;
    // A guest/source can echo arbitrary text on stdout. A negative compile
    // test is admissible only on the compiler's stderr diagnostic channel.
    let haystack = ascii_lower_hex(&invocation.stderr_hex)?;
    for (category, needles) in backend_rules {
        for needle in needles {
            let encoded = hex(needle.to_ascii_lowercase().as_bytes());
            if haystack.contains(&encoded) {
                return Some(category.clone());
            }
        }
    }
    None
}

const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;

fn read_source_text(path: &Path) -> Result<String, String> {
    // Parser/import discovery must obey the same admission rules as hashing.
    // A late FIFO or symlink substitution must not block or escape the audit.
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK);
    }
    let file = options.open(path)
        .map_err(|e| format!("failed to open source text: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err("source text must be a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut bytes)
        .map_err(|e| format!("source text read failed: {e}"))?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("source text changed size or exceeded admission limit".into());
    }
    String::from_utf8(bytes).map_err(|e| format!("source is not UTF-8: {e}"))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    // Source admission must not load unbounded compiler-controlled files,
    // follow a replaced symlink, or block on a FIFO after a preflight race.
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK);
    }
    let mut file = options.open(path)
        .map_err(|e| format!("failed to open source for SHA-256: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err("source must be a bounded regular file".into());
    }
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 16384];
    let mut count = 0u64;
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("source SHA-256 read failed: {e}"))?;
        if n == 0 { break; }
        count += n as u64;
        if count > MAX_SOURCE_BYTES {
            return Err("source exceeded bounded SHA-256 read limit".into());
        }
        hasher.update(&buf[..n]);
    }
    if count != metadata.len() {
        return Err("source changed size during SHA-256 admission".into());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> Result<String, String> {
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

// Walk the lexical path before canonicalization. Checking only the leaf
// permits a symlinked ancestor to redirect a same-byte import after inventory.
fn no_symlink_source_path(root: &Path, original: &Path) -> Result<PathBuf, String> {
    let rel = original.strip_prefix(root)
        .map_err(|_| format!("source path outside corpus root: {}", original.display()))?;
    let mut current = root.to_path_buf();
    for component in rel.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if current == root {
                    return Err("relative import escapes corpus root".into());
                }
                current.pop();
            }
            std::path::Component::Normal(part) => {
                current.push(part);
                let meta = fs::symlink_metadata(&current)
                    .map_err(|e| format!("untrusted source path {}: {e}", current.display()))?;
                if meta.file_type().is_symlink() {
                    return Err(format!("symlinked source path component: {}", current.display()));
                }
            }
            _ => return Err("invalid source path component".into()),
        }
    }
    let meta = fs::symlink_metadata(&current).map_err(|e| e.to_string())?;
    if !meta.file_type().is_file() {
        return Err("source must be a regular file".into());
    }
    let canonical = current.canonicalize().map_err(|e| e.to_string())?;
    if !canonical.starts_with(root) {
        return Err("resolved source escaped corpus root".into());
    }
    Ok(canonical)
}

fn safe_file(root: &Path, path_text: &str) -> Result<PathBuf, String> {
    // Reject absolute/parent-traversing manifest paths and symlinked files
    // before canonicalization erases the original source entry's provenance.
    let relative = Path::new(path_text);
    if !relative.components().all(|part| matches!(part, std::path::Component::Normal(_))) {
        return Err(format!("invalid or traversing source path: {path_text}"));
    }
    no_symlink_source_path(root, &root.join(relative))
}

fn import_target(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("import ") {
        return None;
    }
    let marker = " from \"";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn local_imports(path: &Path, root: &Path) -> Result<Vec<PathBuf>, String> {
    let text = read_source_text(path)?;
    let mut result = Vec::new();
    for target in text.lines().filter_map(import_target) {
        if !target.starts_with('.') {
            continue;
        }
        let mut candidate = path
            .parent()
            .ok_or_else(|| format!("source has no parent: {}", path.display()))?
            .join(&target);
        if candidate.extension().is_none() {
            // A new AttoLang .atto source resolves extensionless imports to
            // .atto, while legacy .ores files retain their historical behavior.
            let extension = path.extension().and_then(|ext| ext.to_str())
                .ok_or_else(|| format!("unknown importing source extension: {}", path.display()))?;
            if !matches!(extension, "atto" | "ores") {
                return Err(format!("unsupported importing source extension: {extension}"));
            }
            candidate.set_extension(extension);
        }
        if !matches!(candidate.extension().and_then(|ext| ext.to_str()), Some("atto" | "ores")) {
            return Err(format!("local import is not an .atto/.ores source file: {target}"));
        }
        // Check EVERY lexical ancestor, not merely the last source component.
        // Canonicalization alone would erase the reviewed path's provenance.
        let candidate = no_symlink_source_path(root, &candidate)
            .map_err(|e| format!("{} imports {target}: {e}", path.display()))?;
        result.push(candidate);
    }
    Ok(result)
}

fn reachable_graph(entry: &Path, root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![entry.to_path_buf()];
    let mut seen = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        pending.extend(local_imports(&current, root)?);
    }
    Ok(seen.into_iter().collect())
}

fn relative_path(path: &Path, root: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|e| e.to_string())
}

fn fingerprint_graph(paths: &[PathBuf], root: &Path) -> Result<String, String> {
    let mut payload = Vec::new();
    for path in paths {
        let rel = relative_path(path, root)?;
        payload.extend_from_slice(rel.as_bytes());
        payload.push(0);
        payload.extend_from_slice(sha256_file(path)?.as_bytes());
        payload.push(b'\n');
    }
    sha256_bytes(&payload)
}

fn lowercase_kebab(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn validate_backend_policy(policy: &LegacyPolicy) -> Result<(), String> {
    let required: BTreeSet<_> = policy.target_backends.iter().map(String::as_str).collect();
    let optional: BTreeSet<_> = policy.optional_backends.iter().map(String::as_str).collect();
    let supported: BTreeSet<_> = BACKENDS.iter().map(|(backend, _)| *backend).collect();
    if required.len() != policy.target_backends.len()
        || optional.len() != policy.optional_backends.len()
        || !required.is_disjoint(&optional)
        || required.union(&optional).copied().collect::<BTreeSet<_>>() != supported
        || required != ["llvm", "java", "wasm", "dotnet", "forge", "cranelift"].into_iter().collect()
        || optional != ["js"].into_iter().collect()
    {
        return Err("required backends must be llvm/java/wasm/dotnet/forge/cranelift; js optional".into());
    }
    Ok(())
}

fn validate_legacy_cases(cases: &[LegacyCase]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for case in cases {
        if !ids.insert(case.id.as_str()) || !names.insert(case.name.as_str())
            || !paths.insert(case.path.as_str())
        {
            return Err(format!("duplicate parity case identity: {}", case.id));
        }
        if case.id.is_empty() || !case.id.bytes().all(|b| b.is_ascii_digit())
            || !lowercase_kebab(&case.name)
            || case.path != format!("programs/{}-{}.ores", case.id, case.name)
        {
            return Err(format!("invalid parity case identifier/path: {}", case.id));
        }
        if !case.required_on_all_backends || case.features.is_empty()
            || case.features.iter().any(|f| f.trim().is_empty())
        {
            return Err(format!("incomplete mandatory parity contract: {}", case.id));
        }
        if !matches!(
            case.validation_status.as_str(),
            "candidate" | "reference_fixture" | "reference_derived"
                | "reference_contract" | "reference_test" | "parser_grounded"
                | "validated" | "unvalidated"
        ) {
            return Err(format!("invalid parity maturity: {}", case.id));
        }
        match case.expected_kind.as_str() {
            "exit" if case.expected_value.is_some()
                && case.expected_diagnostic_category.is_none() => {}
            "compile_reject" if case.expected_value.is_none()
                && case.expected_diagnostic_category.as_ref().is_some_and(|v| !v.is_empty()) => {}
            _ => return Err(format!("invalid expected outcome/diagnostic: {}", case.id)),
        }
    }
    Ok(())
}

fn load_cases(root: &Path) -> Result<(Vec<TestCase>, Corpus), String> {
    let legacy: LegacyManifest = serde_json::from_str(
        &read_source_text(&root.join("manifest.json"))?,
    )
    .map_err(|e| e.to_string())?;
    validate_backend_policy(&legacy.policy)?;
    validate_legacy_cases(&legacy.cases)?;

    let mut cases = Vec::new();
    for case in &legacy.cases {
        let source = safe_file(root, &case.path)?;
        cases.push(TestCase {
            suite: "legacy".into(),
            case_id: case.id.clone(),
            name: case.name.clone(),
            source_sha256: sha256_file(&source)?,
            source,
            graph_sha256: None,
            graph_files: None,
            expected_kind: case.expected_kind.clone(),
            expected_value: case.expected_value,
            expected_diagnostic_category: case.expected_diagnostic_category.clone(),
            expected_stdout_prefix: None,
            validation_status: case.validation_status.clone(),
            features: case.features.clone(),
        });
    }

    let checks_manifest: ChecksManifest = serde_json::from_str(
        &read_source_text(&root.join("checks/manifest.json"))?,
    )
    .map_err(|e| e.to_string())?;
    let mut manifest_names = BTreeSet::new();
    let mut negative_probes = 0usize;

    for item in &checks_manifest.checks {
        if !manifest_names.insert(item.name.clone()) {
            return Err(format!("duplicate check name: {}", item.name));
        }
        if !lowercase_kebab(&item.name) {
            return Err(format!(
                "check name must be lowercase kebab-case: {}",
                item.name
            ));
        }
        let expectation_path = safe_file(root, &item.expectation)?;
        let expectation: Expectation = serde_json::from_str(
            &read_source_text(&expectation_path)?,
        )
        .map_err(|e| format!("{}: {e}", item.expectation))?;
        if expectation.name != item.name {
            return Err(format!("check name mismatch: {}", item.name));
        }

        let check_dir = root
            .join("checks")
            .join(&item.name)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let entry = safe_file(root, &expectation.entry)?;
        if entry.parent() != Some(check_dir.as_path()) {
            return Err(format!(
                "entry is outside its check directory: {}",
                expectation.entry
            ));
        }

        let direct_imports: BTreeSet<PathBuf> = local_imports(&entry, root)?.into_iter().collect();
        if direct_imports.len() != expectation.imported_files {
            return Err(format!(
                "{}: main imports {} local files, expected {}",
                item.name,
                direct_imports.len(),
                expectation.imported_files
            ));
        }
        let graph = reachable_graph(&entry, root)?;
        if graph.len() < 4 {
            return Err(format!(
                "{}: reachable source graph must contain at least four files",
                item.name
            ));
        }
        let graph_files = graph
            .iter()
            .map(|p| relative_path(p, root))
            .collect::<Result<Vec<_>, _>>()?;
        let graph_sha256 = fingerprint_graph(&graph, root)?;
        cases.push(TestCase {
            suite: "checks".into(),
            case_id: item.name.clone(),
            name: item.name.clone(),
            source_sha256: sha256_file(&entry)?,
            source: entry,
            graph_sha256: Some(graph_sha256),
            graph_files: Some(graph_files),
            expected_kind: "exit".into(),
            expected_value: Some(0),
            expected_diagnostic_category: None,
            expected_stdout_prefix: Some(expectation.stable_stdout_prefix.clone()),
            validation_status: if expectation.verified {
                "validated".into()
            } else {
                "unvalidated".into()
            },
            features: expectation.features.clone(),
        });

        for (index, negative) in expectation.negative_cases.iter().enumerate() {
            let source = safe_file(root, &negative.path)?;
            if source.parent() != Some(check_dir.as_path()) {
                return Err(format!(
                    "negative probe is outside its check directory: {}",
                    negative.path
                ));
            }
            negative_probes += 1;
            let mut negative_features = expectation.features.clone();
            negative_features.push("negative-probe".into());
            cases.push(TestCase {
                suite: "checks".into(),
                case_id: format!("{}/negative-{}", item.name, index + 1),
                name: format!("{}-negative-{}", item.name, index + 1),
                source_sha256: sha256_file(&source)?,
                source,
                graph_sha256: None,
                graph_files: None,
                expected_kind: "compile_reject".into(),
                expected_value: None,
                expected_diagnostic_category: negative.expected_diagnostic_category.clone(),
                expected_stdout_prefix: None,
                validation_status: "unvalidated".into(),
                features: negative_features,
            });
        }
    }

    let mut actual_names = BTreeSet::new();
    for entry in fs::read_dir(root.join("checks")).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        if entry.path().join("expectation.json").is_file() {
            actual_names.insert(entry.file_name().to_string_lossy().to_string());
        }
    }
    if actual_names != manifest_names {
        return Err(
            "checks/manifest.json does not exactly enumerate expectation-bearing check directories"
                .into(),
        );
    }

    let corpus = Corpus {
        legacy_cases: legacy.cases.len(),
        multifile_checks: checks_manifest.checks.len(),
        negative_probes,
        total_cases: cases.len(),
    };
    Ok((cases, corpus))
}

// Hash the opened artifact rather than relying on its name or compiler stdout.
// The consumer (e.g. Wasm host) must independently verify this exact digest.
const MAX_ARTIFACT_BYTES: u64 = 128 * 1024 * 1024;

fn artifact_sha256(path: &Path) -> Result<String, String> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK);
    }
    let mut file = options
        .open(path)
        .map_err(|e| format!("compiled artifact unavailable: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_ARTIFACT_BYTES {
        return Err("compiled artifact must be a bounded nonempty regular file".into());
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 16 * 1024];
    let mut count = 0u64;
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > MAX_ARTIFACT_BYTES {
            return Err("compiled artifact exceeded bounded read limit".into());
        }
        hasher.update(&buffer[..n]);
    }
    if count != metadata.len() {
        return Err("compiled artifact changed during hashing".into());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn expand(
    template: &str,
    source: &Path,
    artifact: &Path,
    workdir: &Path,
    artifact_digest: Option<&str>,
) -> Result<Vec<String>, String> {
    if template.contains("{artifact_sha256}") && artifact_digest.is_none() {
        return Err("artifact SHA-256 requested before a verified compile".into());
    }
    let parts = shell_words::split(template).map_err(|e| e.to_string())?;
    Ok(parts
        .into_iter()
        .map(|p| {
            p.replace("{artifact_sha256}", artifact_digest.unwrap_or(""))
                .replace("{source}", &source.to_string_lossy())
                .replace("{artifact}", &artifact.to_string_lossy())
                .replace("{workdir}", &workdir.to_string_lossy())
        })
        .collect())
}

// Source compilers and guest programs are not trusted to emit bounded output.
// Drain to EOF even after the capture limit, so a full pipe cannot block a
// child, but never promote truncated diagnostics to parity success.
const MAX_CAPTURE_BYTES: usize = 1024 * 1024;

struct BoundedCapture {
    bytes: Vec<u8>,
    exceeded: bool,
}

fn read_bounded(mut pipe: impl Read) -> std::io::Result<BoundedCapture> {
    let mut bytes = Vec::new();
    let mut exceeded = false;
    let mut chunk = [0u8; 8192];
    loop {
        let count = match pipe.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        let remaining = MAX_CAPTURE_BYTES.saturating_sub(bytes.len());
        let accepted = count.min(remaining);
        bytes.extend_from_slice(&chunk[..accepted]);
        if count > accepted {
            exceeded = true;
        }
    }
    Ok(BoundedCapture { bytes, exceeded })
}

fn invoke(argv: &[String], workdir: &Path, timeout: Duration) -> Invocation {
    if argv.is_empty() {
        return Invocation {
            status: "infrastructure_error".into(),
            exit_code: None,
            stdout_hex: String::new(),
            stderr_hex: String::new(),
            duration_ms: 0,
            error: Some("empty command".into()),
        };
    }
    let start = Instant::now();
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return Invocation {
                status: "infrastructure_error".into(),
                exit_code: None,
                stdout_hex: String::new(),
                stderr_hex: String::new(),
                duration_ms: start.elapsed().as_millis(),
                error: Some(error.to_string()),
            }
        }
    };
    // Independently drain both pipes with fixed storage while the child runs.
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let stdout_reader = std::thread::spawn(move || read_bounded(stdout));
    let stderr_reader = std::thread::spawn(move || read_bounded(stderr));
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if start.elapsed() >= timeout => {
                timed_out = true;
                terminate(&mut child);
                break child.wait();
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                terminate(&mut child);
                let _ = child.wait();
                break Err(error);
            }
        }
    };
    let stdout = stdout_reader.join().expect("stdout reader panicked");
    let stderr = stderr_reader.join().expect("stderr reader panicked");
    match (status, stdout, stderr) {
        (Ok(_), Ok(stdout), Ok(stderr)) if stdout.exceeded || stderr.exceeded => Invocation {
            status: "output_limit_exceeded".into(),
            exit_code: None,
            stdout_hex: hex(&stdout.bytes),
            stderr_hex: hex(&stderr.bytes),
            duration_ms: start.elapsed().as_millis(),
            error: Some(format!(
                "compiler or guest output exceeded {} bytes per stream; truncated output cannot certify parity",
                MAX_CAPTURE_BYTES
            )),
        },
        (Ok(status), Ok(stdout), Ok(stderr)) => Invocation {
            status: if timed_out { "timeout" } else { "completed" }.into(),
            exit_code: if timed_out { None } else { status.code() },
            stdout_hex: hex(&stdout.bytes),
            stderr_hex: hex(&stderr.bytes),
            duration_ms: start.elapsed().as_millis(),
            error: None,
        },
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => Invocation {
            status: "infrastructure_error".into(),
            exit_code: None,
            stdout_hex: String::new(),
            stderr_hex: String::new(),
            duration_ms: start.elapsed().as_millis(),
            error: Some(error.to_string()),
        },
    }
}

fn terminate(child: &mut std::process::Child) {
    // The adapter launches Java as a child. Kill its isolated process group so
    // a timeout cannot leave Java running or keep the capture pipes open.
    #[cfg(unix)]
    unsafe {
        libc::killpg(child.id() as libc::pid_t, libc::SIGKILL);
    }
    let _ = child.kill();
}

fn safe_temp_component(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn temp_dir(case_id: &str, backend: &str) -> Result<PathBuf, String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_WORKDIR: AtomicU64 = AtomicU64::new(0);
    // Do not delete a predictable pre-existing /tmp path. A prior runner or
    // unrelated user might own it; deleting it before creating a workspace
    // could destroy files or follow malicious directory structures.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    for _ in 0..64 {
        let sequence = NEXT_WORKDIR.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "ores-parity-{}-{nonce:x}-{sequence:x}-{}-{}",
            std::process::id(),
            safe_temp_component(case_id),
            safe_temp_component(backend)
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("cannot create isolated parity workdir: {error}")),
        }
    }
    Err("cannot allocate a unique parity workdir after 64 attempts".into())
}

fn case_sources_match(case: &TestCase, root: &Path) -> Result<bool, String> {
    // A successful preflight cannot authorize a later symlink swap.
    // Recheck the physical entry before each compile/run evidence boundary.
    let metadata = match fs::symlink_metadata(&case.source) {
        Ok(value) => value,
        Err(_) => return Ok(false),
    };
    if !metadata.file_type().is_file() {
        return Ok(false);
    }
    if sha256_file(&case.source)? != case.source_sha256 {
        return Ok(false);
    }
    if let Some(expected_graph_sha256) = case.graph_sha256.as_ref() {
        let actual_graph = reachable_graph(&case.source, root)?;
        let actual_files = actual_graph
            .iter()
            .map(|path| relative_path(path, root))
            .collect::<Result<Vec<_>, _>>()?;
        if case.graph_files.as_ref() != Some(&actual_files) {
            return Ok(false);
        }
        if fingerprint_graph(&actual_graph, root)? != *expected_graph_sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn evaluate_case(
    case: &TestCase,
    root: &Path,
    diagnostic_rules: &DiagnosticRules,
    backend: &str,
    env_name: &str,
    execute: bool,
    allow_unvalidated: bool,
    timeout: Duration,
) -> Result<Record, String> {
    let mut record = Record {
        suite: case.suite.clone(),
        case_id: case.case_id.clone(),
        name: case.name.clone(),
        backend: backend.into(),
        source_sha256: case.source_sha256.clone(),
        graph_sha256: case.graph_sha256.clone(),
        graph_files: case.graph_files.clone(),
        validation_status: case.validation_status.clone(),
        features: case.features.clone(),
        expected_diagnostic_category: case.expected_diagnostic_category.clone(),
        actual_diagnostic_category: None,
        status: "not_run".into(),
        compile: None,
        run: None,
    };
    if !execute {
        return Ok(record);
    }
    if case.validation_status != "validated" && !allow_unvalidated {
        record.status = "unvalidated".into();
        return Ok(record);
    }

    let prefix = format!("ORES_PARITY_{env_name}");
    let compile_template = env::var(format!("{prefix}_COMPILE")).ok();
    let run_template = env::var(format!("{prefix}_RUN")).ok();
    if compile_template.is_none() || (case.expected_kind == "exit" && run_template.is_none()) {
        record.status = "backend_not_configured".into();
        return Ok(record);
    }

    // Detect corpus mutation or a changed recursive import graph after the
    // inventory snapshot. Such a run has no trustworthy source identity.
    if !case_sources_match(case, root)? {
        record.status = "source_integrity_failure".into();
        return Ok(record);
    }
    let workdir = temp_dir(&case.case_id, backend)?;
    let artifact = workdir.join("program");
    let compile_argv = expand(
        compile_template.as_ref().expect("checked above"),
        &case.source,
        &artifact,
        &workdir,
        None,
    )?;
    let compile = invoke(&compile_argv, &workdir, timeout);
    let compile_status = compile.status.clone();
    let compile_exit = compile.exit_code;
    record.compile = Some(compile);
    if !case_sources_match(case, root)? {
        record.status = "source_integrity_failure".into();
        let _ = fs::remove_dir_all(&workdir);
        return Ok(record);
    }

    if compile_status != "completed" || compile_exit.is_none() || compile_exit == Some(125) {
        record.status = "compile_infrastructure_failure".into();
    } else if case.expected_kind == "compile_reject" {
        if compile_exit == Some(0) {
            record.status = "unexpected_compile_success".into();
        } else if fs::read_dir(&workdir)
            .map_err(|e| e.to_string())?
            .next()
            .transpose()
            .map_err(|e| e.to_string())?
            .is_some()
        {
            // No emitted guest binaries, partial modules, symlinks, or
            // published release pointers may survive a rejected source.
            record.status = "rejected_with_artifacts".into();
        } else {
            record.actual_diagnostic_category = record
                .compile
                .as_ref()
                .and_then(|c| classify_diagnostic(diagnostic_rules, backend, c));
            record.status = match (
                case.expected_diagnostic_category.as_deref(),
                record.actual_diagnostic_category.as_deref(),
            ) {
                (Some(expected), Some(actual)) if expected == actual => {
                    if case.validation_status == "validated" {
                        "expected_reject".into()
                    } else {
                        "exploratory_expected_reject_not_parity_pass".into()
                    }
                }
                (_, Some(_)) => "wrong_diagnostic".into(),
                _ => "rejection_unclassified".into(),
            };
        }
    } else if compile_exit != Some(0) {
        record.actual_diagnostic_category = record
            .compile
            .as_ref()
            .and_then(|c| classify_diagnostic(diagnostic_rules, backend, c));
        let select_case = case.features.iter().any(|feature| {
            matches!(
                feature.as_str(),
                "select" | "do-select" | "nb-select" | "do-nb-select"
            )
        });
        record.status = if select_case
            && record.actual_diagnostic_category.as_deref() == Some("unsupported_select")
        {
            "intentional_capability_reject".into()
        } else {
            "compile_failure".into()
        };
    } else {
        let template = run_template.as_ref().expect("checked above");
        // The Wasm host requires an exact output digest. A run adapter that
        // omits the digest is not a legitimate Wasm parity configuration.
        let requires_digest = template.contains("{artifact_sha256}");
        if backend == "wasm" && !requires_digest {
            record.status = "runtime_infrastructure_failure".into();
            let _ = fs::remove_dir_all(&workdir);
            return Ok(record);
        }
        let digest = if requires_digest {
            match artifact_sha256(&artifact) {
                Ok(hash) => Some(hash),
                Err(message) => {
                    record.status = "runtime_infrastructure_failure".into();
                    record.run = Some(Invocation {
                        status: "infrastructure_error".into(),
                        exit_code: None, stdout_hex: String::new(),
                        stderr_hex: String::new(), duration_ms: 0,
                        error: Some(message),
                    });
                    let _ = fs::remove_dir_all(&workdir);
                    return Ok(record);
                }
            }
        } else {
            None
        };
        let run_argv = expand(
            template,
            &case.source,
            &artifact,
            &workdir,
            digest.as_deref(),
        )?;
        let run = invoke(&run_argv, &workdir, timeout);
        let run_status = run.status.clone();
        let run_exit = run.exit_code;
        let stdout_hex = run.stdout_hex.clone();
        record.run = Some(run);
        record.status = if run_status != "completed" || run_exit.is_none() || run_exit == Some(125)
        {
            "runtime_infrastructure_failure".into()
        } else if run_exit != case.expected_value {
            "wrong_result".into()
        } else if let Some(prefix) = &case.expected_stdout_prefix {
            if !stdout_hex.starts_with(&hex(prefix.as_bytes())) {
                "wrong_stdout".into()
            } else if case.validation_status == "validated" {
                "pass".into()
            } else {
                "exploratory_match_not_parity_pass".into()
            }
        } else if case.validation_status == "validated" {
            "pass".into()
        } else {
            "exploratory_match_not_parity_pass".into()
        };
        if let Some(expected_digest) = digest.as_deref() {
            if artifact_sha256(&artifact).ok().as_deref() != Some(expected_digest) {
                record.status = "artifact_integrity_failure".into();
            }
        }
    }
    if !case_sources_match(case, root)? {
        record.status = "source_integrity_failure".into();
    }
    if let Err(error) = fs::remove_dir_all(&workdir) {
        eprintln!("parity workdir cleanup failed ({}): {error}", workdir.display());
        record.status = "cleanup_infrastructure_failure".into();
    }
    Ok(record)
}

fn validate_cli_args(args: &[String]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    let mut index = 1usize;
    while index < args.len() {
        let option = args[index].as_str();
        if !matches!(
            option,
            "--execute" | "--allow-unvalidated" | "--backend" | "--output" | "--timeout"
        ) {
            return Err(format!("unknown CLI option: {option}"));
        }
        if !seen.insert(option) {
            return Err(format!("duplicate CLI option: {option}"));
        }
        if matches!(option, "--backend" | "--output" | "--timeout") {
            if args.get(index + 1).is_none_or(|value| value.starts_with("--")) {
                return Err(format!("{option} requires a value"));
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    if seen.contains("--allow-unvalidated") && !seen.contains("--execute") {
        return Err("--allow-unvalidated requires --execute".into());
    }
    Ok(())
}

fn is_verified_execution_status(status: &str) -> bool {
    matches!(status, "pass" | "expected_reject")
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    validate_cli_args(&args)?;
    let execute = args.iter().any(|arg| arg == "--execute");
    let allow_unvalidated = args.iter().any(|arg| arg == "--allow-unvalidated");
    let selected_backend = args
        .iter()
        .position(|arg| arg == "--backend")
        .map(|index| {
            args.get(index + 1)
                .filter(|name| BACKENDS.iter().any(|(backend, _)| backend == name))
                .cloned()
                .ok_or_else(|| "--backend requires llvm, java, wasm, dotnet, forge, cranelift, or js".to_string())
        })
        .transpose()?;
    let output = args
        .windows(2)
        .find(|window| window[0] == "--output")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| "parity-results.json".into());
    let timeout_seconds: u64 = args
        .windows(2)
        .find(|window| window[0] == "--timeout")
        .map(|window| window[1].parse::<u64>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(30);
    if timeout_seconds == 0 {
        return Err("--timeout must be positive".into());
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let (cases, corpus) = load_cases(&root)?;
    let diagnostic_rules: DiagnosticRules = serde_json::from_str(
        &read_source_text(&root.join("diagnostics.json"))?,
    )
    .map_err(|e| e.to_string())?;
    let timeout = Duration::from_secs(timeout_seconds);
    let mut records = Vec::new();
    for case in &cases {
        for &(backend, env_name) in &BACKENDS {
            if selected_backend.is_none() && backend == "js" {
                continue;
            }
            if selected_backend
                .as_deref()
                .is_some_and(|selected| selected != backend)
            {
                continue;
            }
            records.push(evaluate_case(
                case,
                &root,
                &diagnostic_rules,
                backend,
                env_name,
                execute,
                allow_unvalidated,
                timeout,
            )?);
        }
    }

    let mut summary = BTreeMap::new();
    for record in &records {
        *summary.entry(record.status.clone()).or_insert(0) += 1;
    }
    let report = Report {
        schema_version: 2,
        executed: execute,
        cases: cases.len(),
        corpus,
        backends: BACKENDS
            .iter()
            .filter(|entry| {
                (selected_backend.is_some() || entry.0 != "js")
                    && selected_backend
                        .as_deref()
                        .is_none_or(|name| name == entry.0)
            })
            .map(|entry| entry.0.to_string())
            .collect(),
        records,
        summary,
    };
    fs::write(
        &output,
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string(&report.summary).map_err(|e| e.to_string())?
    );

    // --execute is a certification attempt, not an inventory command. Even
    // exploratory matches, intentional unsupported cases and unknown future
    // statuses must return a failing process code. Inventory mode remains
    // explicitly non-executing and may exit zero.
    let hard_fail = execute && (report.records.is_empty()
        || report.records.iter().any(|record| {
            !is_verified_execution_status(&record.status)
        }));
    if hard_fail {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ascii_lower_hex, classify_diagnostic, hex, import_target, lowercase_kebab,
        validate_backend_policy, validate_cli_args, Invocation, LegacyPolicy,
    };
    use std::collections::BTreeMap;

    #[test]
    fn diagnostic_classification_is_ascii_case_insensitive() {
        let mut backend = BTreeMap::new();
        backend.insert(
            "unsupported_select".to_string(),
            vec!["static select requires checked channel/future arbitration".to_string()],
        );
        let mut rules = BTreeMap::new();
        rules.insert("dotnet".to_string(), backend);
        let invocation = Invocation {
            status: "completed".into(),
            exit_code: Some(1),
            stdout_hex: String::new(),
            stderr_hex: hex(b"Static select requires checked Channel/Future arbitration in .NET"),
            duration_ms: 1,
            error: None,
        };
        assert_eq!(
            classify_diagnostic(&rules, "dotnet", &invocation).as_deref(),
            Some("unsupported_select")
        );
    }

    #[test]
    fn stdout_spoofing_cannot_classify_compiler_diagnostics() {
        let mut backend = BTreeMap::new();
        backend.insert("integer_range".to_string(), vec!["signed 64-bit".to_string()]);
        let mut rules = BTreeMap::new();
        rules.insert("java".to_string(), backend);
        let mut invocation = Invocation {
            status: "completed".into(),
            exit_code: Some(1),
            stdout_hex: hex(b"integer literal overflows signed 64-bit"),
            stderr_hex: hex(b"unrelated compiler failure"),
            duration_ms: 0,
            error: None,
        };
        assert_eq!(classify_diagnostic(&rules, "java", &invocation), None);
        invocation.stderr_hex = hex(b"integer literal overflows signed 64-bit");
        assert_eq!(
            classify_diagnostic(&rules, "java", &invocation).as_deref(),
            Some("integer_range")
        );
    }

    #[test]
    fn exploratory_unknown_and_source_only_results_cannot_certify_execution() {
        assert!(super::is_verified_execution_status("pass"));
        assert!(super::is_verified_execution_status("expected_reject"));
        for status in [
            "not_run", "unvalidated", "exploratory_match_not_parity_pass",
            "exploratory_expected_reject_not_parity_pass", "intentional_capability_reject",
            "backend_not_configured", "compile_infrastructure_failure",
            "runtime_infrastructure_failure", "unknown_future_status",
        ] {
            assert!(!super::is_verified_execution_status(status), "{status}");
        }
    }

    #[test]
    fn manifest_backend_roster_is_exact_and_unique() {
        let mut roster = LegacyPolicy {
            target_backends: vec!["llvm", "java", "wasm", "dotnet", "forge", "cranelift"]
                .into_iter().map(str::to_string).collect(),
            optional_backends: vec!["js".into()],
        };
        assert!(validate_backend_policy(&roster).is_ok());
        roster.target_backends.pop();
        assert!(validate_backend_policy(&roster).is_err());
        roster.target_backends.push("java".into());
        assert!(validate_backend_policy(&roster).is_err());
        roster.target_backends.pop();
        roster.target_backends.push("cranelift".into());
        roster.optional_backends.push("forge".into());
        assert!(validate_backend_policy(&roster).is_err());
    }

    #[test]
    fn cli_rejects_unknown_duplicate_or_missing_options() {
        let argv = |options: &[&str]| {
            std::iter::once("runner".to_string())
                .chain(options.iter().map(|arg| arg.to_string()))
                .collect::<Vec<_>>()
        };
        assert!(validate_cli_args(&argv(&["--backend", "cranelift", "--execute"])).is_ok());
        assert!(validate_cli_args(&argv(&["--backend", "java", "--backend", "cranelift"])).is_err());
        assert!(validate_cli_args(&argv(&["--execute", "--execute"])).is_err());
        assert!(validate_cli_args(&argv(&["--backend"])).is_err());
        assert!(validate_cli_args(&argv(&["--backend", "--execute"])).is_err());
        assert!(validate_cli_args(&argv(&["--not-a-real-mode"])).is_err());
        assert!(validate_cli_args(&argv(&["--allow-unvalidated"])).is_err());
    }

    #[test]
    fn workdirs_are_distinct_and_never_delete_legacy_collisions() {
        let case_id = format!(
            "collision-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let legacy = std::env::temp_dir().join(format!(
            "ores-parity-{}-{}-{}",
            std::process::id(), case_id, "cranelift"
        ));
        std::fs::create_dir(&legacy).unwrap();
        let sentinel = legacy.join("must-survive.txt");
        std::fs::write(&sentinel, b"untouched").unwrap();
        let a = super::temp_dir(&case_id, "cranelift").unwrap();
        let b = super::temp_dir(&case_id, "cranelift").unwrap();
        assert_ne!(a, b);
        assert_ne!(a, legacy);
        assert_ne!(b, legacy);
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"untouched");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in [&a, &b] {
                assert_eq!(std::fs::metadata(path).unwrap().permissions().mode() & 0o777, 0o700);
            }
        }
        std::fs::remove_dir_all(a).unwrap();
        std::fs::remove_dir_all(b).unwrap();
        std::fs::remove_file(sentinel).unwrap();
        std::fs::remove_dir(legacy).unwrap();
    }

    #[test]
    fn workdir_name_components_cannot_escape_temp_root() {
        let path = super::temp_dir("../../outside", "..\\\\evil").unwrap();
        assert_eq!(path.parent(), Some(std::env::temp_dir().as_path()));
        assert!(path.is_dir());
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn source_integrity_detects_changes_to_entry_and_import_graph() {
        let root = super::temp_dir("graph-check", "java").unwrap();
        std::fs::create_dir(root.join("case")).unwrap();
        let entry = root.join("case/main.ores");
        let dep = root.join("case/dependency.ores");
        std::fs::write(&entry, b"import fnc x from \"./dependency.ores\";\n").unwrap();
        std::fs::write(&dep, b"pub fnc x(): int { return 42; }\n").unwrap();
        let graph = super::reachable_graph(&entry, &root).unwrap();
        let case = super::TestCase {
            suite: "test".into(),
            case_id: "local-graph".into(),
            name: "local-graph".into(),
            source: entry.clone(),
            source_sha256: super::sha256_file(&entry).unwrap(),
            graph_sha256: Some(super::fingerprint_graph(&graph, &root).unwrap()),
            graph_files: Some(graph.iter()
                .map(|p| super::relative_path(p, &root).unwrap()).collect()),
            expected_kind: "exit".into(),
            expected_value: Some(42),
            expected_diagnostic_category: None,
            expected_stdout_prefix: None,
            validation_status: "candidate".into(),
            features: vec!["imports".into()],
        };
        assert!(super::case_sources_match(&case, &root).unwrap());
        std::fs::write(&dep, b"pub fnc x(): int { return 43; }\n").unwrap();
        assert!(!super::case_sources_match(&case, &root).unwrap());
        std::fs::write(&dep, b"pub fnc x(): int { return 42; }\n").unwrap();
        assert!(super::case_sources_match(&case, &root).unwrap());
        std::fs::write(&entry, b"pub fnc main(): int { return 42; }\n").unwrap();
        assert!(!super::case_sources_match(&case, &root).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imported_source_text_is_bounded_before_decoding() {
        let root = super::temp_dir("bounded-import-source", "wasm").unwrap();
        let source = root.join("import.ores");
        std::fs::write(&source, "fnc value(): int { return 42; }").unwrap();
        assert!(super::read_source_text(&source).unwrap().contains("return 42"));
        std::fs::write(&source, vec![b'x'; (super::MAX_SOURCE_BYTES + 1) as usize]).unwrap();
        assert!(super::read_source_text(&source).is_err());
        std::fs::write(&source, [0xff, 0xfe]).unwrap();
        assert!(super::read_source_text(&source).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn import_source_read_refuses_symlink_to_regular_file() {
        use std::os::unix::fs::symlink;
        let root = super::temp_dir("import-source-symlink", "net").unwrap();
        let source = root.join("real.ores");
        let alias = root.join("alias.ores");
        std::fs::write(&source, "fnc value(): int { return 42; }").unwrap();
        symlink(&source, &alias).unwrap();
        assert!(super::read_source_text(&alias).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_sha256_is_streamed_and_size_bounded() {
        let root = super::temp_dir("source-hash-budget", "java").unwrap();
        let source = root.join("guest.ores");
        std::fs::write(&source, b"pub fnc main():int{return 42;}").unwrap();
        assert_eq!(super::sha256_file(&source).unwrap().len(), 64);
        std::fs::write(&source, vec![b'x'; (super::MAX_SOURCE_BYTES + 1) as usize]).unwrap();
        assert!(super::sha256_file(&source).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn attolang_extensionless_local_imports_follow_the_entry_extension() {
        let root = super::temp_dir("atto-import", "java").unwrap();
        std::fs::write(root.join("entry.atto"),
            "import fnc answer from \"./answer\";\npub fnc main(): int {return answer();}").unwrap();
        std::fs::write(root.join("answer.atto"), "pub fnc answer(): int {return 42;}").unwrap();
        std::fs::write(root.join("answer.ores"), "pub fnc answer(): int {return 1;}").unwrap();
        let entry = super::safe_file(&root, "entry.atto").unwrap();
        let imports = super::local_imports(&entry, &root).unwrap();
        assert_eq!(imports, vec![root.join("answer.atto")]);
        assert_eq!(super::reachable_graph(&entry, &root).unwrap().len(), 2);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_extensionless_local_imports_keep_ores_resolution() {
        let root = super::temp_dir("ores-import", "llvm").unwrap();
        std::fs::write(root.join("entry.ores"),
            "import fnc answer from \"./answer\";\npub fnc main(): int {return answer();}").unwrap();
        std::fs::write(root.join("answer.ores"), "pub fnc answer(): int {return 42;}").unwrap();
        std::fs::write(root.join("answer.atto"), "pub fnc answer(): int {return 1;}").unwrap();
        let entry = super::safe_file(&root, "entry.ores").unwrap();
        assert_eq!(super::local_imports(&entry, &root).unwrap(), vec![root.join("answer.ores")]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_ancestor_cannot_hide_same_byte_guest_source() {
        use std::os::unix::fs::symlink;
        let root = super::temp_dir("ancestor-symlink", "cranelift").unwrap();
        std::fs::create_dir(root.join("original")).unwrap();
        std::fs::write(root.join("original/guest.atto"),
            "pub fnc main(): int {return 42;}").unwrap();
        symlink(root.join("original"), root.join("alias")).unwrap();
        assert!(super::safe_file(&root, "alias/guest.atto").is_err());
        std::fs::write(root.join("entry.atto"),
            "import fnc guest from \"./alias/guest.atto\";").unwrap();
        let entry = super::safe_file(&root, "entry.atto").unwrap();
        assert!(super::local_imports(&entry, &root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn source_digest_refuses_same_byte_symlink_alias() {
        use std::os::unix::fs::symlink;
        let root = super::temp_dir("source-hash-symlink", "llvm").unwrap();
        let original = root.join("original.ores");
        let link = root.join("alias.ores");
        std::fs::write(&original, b"pub fnc main():int{return 42;}").unwrap();
        symlink(&original, &link).unwrap();
        assert!(super::sha256_file(&link).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn manifest_source_symlink_and_parent_traversal_are_refused() {
        use std::os::unix::fs::symlink;
        let root = super::temp_dir("symlink-inputs", "cranelift").unwrap();
        std::fs::write(root.join("source.ores"), "pub fnc main():int{return 42;}").unwrap();
        symlink("source.ores", root.join("alias.ores")).unwrap();
        assert!(super::safe_file(&root, "source.ores").is_ok());
        assert!(super::safe_file(&root, "alias.ores").is_err());
        assert!(super::safe_file(&root, "../source.ores").is_err());
        assert!(super::safe_file(&root, "/tmp/source.ores").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn changed_regular_source_to_symlink_is_not_valid_evidence() {
        use std::os::unix::fs::symlink;
        let root = super::temp_dir("source-swap", "llvm").unwrap();
        let source = root.join("guest.ores");
        let alternate = root.join("same-bytes.ores");
        std::fs::write(&source, "pub fnc main():int{return 42;}").unwrap();
        std::fs::write(&alternate, "pub fnc main():int{return 42;}").unwrap();
        let test_case = super::TestCase {
            suite: "legacy".into(), case_id: "99".into(), name: "swap".into(),
            source: source.clone(),
            source_sha256: super::sha256_file(&source).unwrap(),
            graph_sha256: None, graph_files: None, expected_kind: "exit".into(),
            expected_value: Some(42), expected_diagnostic_category: None,
            expected_stdout_prefix: None, validation_status: "candidate".into(),
            features: vec!["int".into()],
        };
        assert!(super::case_sources_match(&test_case, &root).unwrap());
        std::fs::remove_file(&source).unwrap();
        symlink(&alternate, &source).unwrap();
        assert!(!super::case_sources_match(&test_case, &root).unwrap(),
            "same-byte symlink substitution must be rejected");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn reachable_source_graph_refuses_symlinked_imports() {
        use std::os::unix::fs::symlink;
        let root = super::temp_dir("symlink-import", "java").unwrap();
        std::fs::write(root.join("real.ores"), "fnc answer():int{return 42;}").unwrap();
        std::fs::write(
            root.join("main.ores"),
            "import fnc answer from \"./alias.ores\";\npub fnc main():int{return answer();}",
        )
        .unwrap();
        symlink("real.ores", root.join("alias.ores")).unwrap();
        let entry = root.join("main.ores");
        assert!(super::local_imports(&entry, &root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manifest_cases_reject_duplicates_and_wrong_expected_result_classes() {
        let make_case = |id: &str, name: &str, kind: &str| super::LegacyCase {
            id: id.into(),
            name: name.into(),
            path: format!("programs/{id}-{name}.ores"),
            expected_kind: kind.into(),
            expected_value: Some(42),
            expected_diagnostic_category: None,
            validation_status: "candidate".into(),
            features: vec!["int".into()],
            required_on_all_backends: true,
        };
        let valid = make_case("90", "test-one", "exit");
        let duplicate = make_case("90", "test-two", "exit");
        assert!(super::validate_legacy_cases(&[valid, duplicate]).is_err());

        let mut invalid = make_case("91", "test-three", "compile_reject");
        assert!(super::validate_legacy_cases(&[invalid]).is_err());
        invalid.expected_value = None;
        invalid.expected_diagnostic_category = Some("type_mismatch".into());
        assert!(super::validate_legacy_cases(&[invalid]).is_ok());
    }

    #[test]
    fn manifest_cases_require_exact_path_and_all_backend_admission() {
        let mut row = super::LegacyCase {
            id: "92".into(),
            name: "test-four".into(),
            path: "programs/92-test-four.ores".into(),
            expected_kind: "exit".into(),
            expected_value: Some(42),
            expected_diagnostic_category: None,
            validation_status: "candidate".into(),
            features: vec!["int".into()],
            required_on_all_backends: true,
        };
        assert!(super::validate_legacy_cases(&[row]).is_ok());
        row.path = "../unreviewed.ores".into();
        assert!(super::validate_legacy_cases(&[row]).is_err());
        row.path = "programs/92-test-four.ores".into();
        row.required_on_all_backends = false;
        assert!(super::validate_legacy_cases(&[row]).is_err());
    }

    #[test]
    fn all_six_backends_unique_and_cranelift_is_first_class() {
        let mut unique = std::collections::BTreeSet::new();
        for &(backend, env_prefix) in &super::BACKENDS {
            assert!(unique.insert(backend), "duplicate backend double-counts");
            assert!(!env_prefix.is_empty());
        }
        assert_eq!(unique.len(), 6);
        for required in ["java", "llvm", "wasm", "dotnet", "cranelift", "js"] {
            assert!(unique.contains(required), "missing backend {required}");
        }
    }

    #[test]
    fn ascii_lower_hex_rejects_malformed_capture() {
        assert!(ascii_lower_hex("0").is_none());
        assert!(ascii_lower_hex("zz").is_none());
    }

    #[test]
    fn parses_relative_import_target() {
        assert_eq!(
            import_target(r#"import fnc value from "./dep.ores";"#),
            Some("./dep.ores".into())
        );
        assert_eq!(import_target("pub fnc main(): void { return; }"), None);
    }

    #[test]
    fn enforces_lowercase_kebab_names() {
        assert!(lowercase_kebab("actor-system"));
        assert!(lowercase_kebab("nb-match"));
        assert!(!lowercase_kebab("Actor-System"));
        assert!(!lowercase_kebab("actor_system"));
        assert!(!lowercase_kebab("actor--system"));
    }

    #[test]
    fn hex_encoding_is_stable() {
        assert_eq!(hex(b"PARITY_OK"), "5041524954595f4f4b");
    }

    fn artifact_test_dir() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ores-parity-artifact-{}-{id}", std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn artifact_hash_is_bound_to_compiled_file_content() {
        let root = artifact_test_dir();
        let artifact = root.join("program");
        std::fs::write(&artifact, b"wasm-build-a").unwrap();
        let a = super::artifact_sha256(&artifact).unwrap();
        assert_eq!(a.len(), 64);
        let argv = super::expand(
            "runner --sha256={artifact_sha256} --file={artifact}",
            &artifact,
            &artifact,
            &root,
            Some(&a),
        ).unwrap();
        assert_eq!(argv[1], format!("--sha256={a}"));
        assert_eq!(argv[2], format!("--file={}", artifact.display()));
        std::fs::write(&artifact, b"wasm-build-b").unwrap();
        let b = super::artifact_sha256(&artifact).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn digest_cannot_be_available_before_compile() {
        let root = artifact_test_dir();
        let artifact = root.join("program");
        let result = super::expand(
            "compile {artifact_sha256}",
            &artifact,
            &artifact,
            &root,
            None,
        );
        assert!(result.unwrap_err().contains("before a verified compile"));
        assert!(super::artifact_sha256(&artifact).is_err());
        std::fs::write(&artifact, b"").unwrap();
        assert!(super::artifact_sha256(&artifact).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn digest_admission_rejects_symlink_and_directory_artifacts() {
        use std::os::unix::fs::symlink;
        let root = artifact_test_dir();
        let original = root.join("original");
        std::fs::write(&original, b"verified").unwrap();
        let symlink_path = root.join("program");
        symlink(&original, &symlink_path).unwrap();
        assert!(super::artifact_sha256(&symlink_path).is_err());
        assert!(super::artifact_sha256(&root).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn drains_output_larger_than_pipe_buffers() {
        let result = super::invoke(
            &[
                "sh".into(),
                "-c".into(),
                "head -c 131072 /dev/zero; head -c 131072 /dev/zero >&2".into(),
            ],
            &std::env::temp_dir(),
            std::time::Duration::from_secs(5),
        );
        assert_eq!(result.status, "completed");
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.stdout_hex.len(), 262144);
        assert_eq!(result.stderr_hex.len(), 262144);
    }

    #[test]
    fn bounded_capture_drains_and_rejects_oversized_bytes() {
        let input = vec![b'x'; super::MAX_CAPTURE_BYTES + 8193];
        let capture = super::read_bounded(std::io::Cursor::new(input)).unwrap();
        assert_eq!(capture.bytes.len(), super::MAX_CAPTURE_BYTES);
        assert!(capture.exceeded);
        let exact_input = vec![b'y'; super::MAX_CAPTURE_BYTES];
        let exact = super::read_bounded(std::io::Cursor::new(exact_input)).unwrap();
        assert!(!exact.exceeded);
        assert_eq!(exact.bytes.len(), super::MAX_CAPTURE_BYTES);
    }

    #[cfg(unix)]
    #[test]
    fn oversized_compiler_stdout_and_stderr_never_certify_parity() {
        for command in [
            "head -c 1048577 /dev/zero",
            "head -c 1048577 /dev/zero >&2",
        ] {
            let result = super::invoke(
                &["sh".into(), "-c".into(), command.into()],
                &std::env::temp_dir(),
                std::time::Duration::from_secs(5),
            );
            assert_eq!(result.status, "output_limit_exceeded");
            assert_eq!(result.exit_code, None);
            assert!(result
                .error
                .as_deref()
                .unwrap()
                .contains("cannot certify parity"));
            assert!(result.stdout_hex.len() <= 2 * super::MAX_CAPTURE_BYTES);
            assert!(result.stderr_hex.len() <= 2 * super::MAX_CAPTURE_BYTES);
        }
    }

    #[cfg(unix)]
    #[test]
    fn timeout_terminates_launcher_children_and_retains_diagnostics() {
        let start = std::time::Instant::now();
        let result = super::invoke(
            &[
                "sh".into(),
                "-c".into(),
                "echo before-timeout >&2; sleep 10".into(),
            ],
            &std::env::temp_dir(),
            std::time::Duration::from_millis(100),
        );
        assert_eq!(result.status, "timeout");
        assert!(result.stderr_hex.contains(&hex(b"before-timeout")));
        assert!(start.elapsed() < std::time::Duration::from_secs(3));
    }
}
