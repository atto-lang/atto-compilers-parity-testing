//! Verifies *downloaded artifact bytes*, not just a self-reported API digest.
//! SHA-256 and ZIP processing are entirely Rust. This is not proof of guest execution.
use crate::decode_strict_json;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub const MAX_ARTIFACT_ZIP_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_RECEIPT_BYTES: usize = 1_048_576;
pub const MAX_FIXTURE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedReceiptPolicy {
    pub repository: String,
    pub run_id: u64,
    pub job_id: u64,
    pub source_sha: String,
    pub tree_sha: String,
    pub backend: String,
    pub feature_id: String,
    pub case_set_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u32,
    repository: String,
    run_id: u64,
    job_id: u64,
    source_sha: String,
    tree_sha: String,
    backend: String,
    feature_id: String,
    case_set_sha256: String,
    status: String,
    execution_target: String,
    executed_cases: u64,
    failed_cases: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedArchive {
    pub artifact_bytes_verified: bool,
    pub fixture_bytes_verified: bool,
    /// The JSON claims a guest test ran. This is NOT independent guest evidence.
    pub receipt_claim_consistent: bool,
    pub guest_execution_verified: bool,
    pub archive_sha256: String,
    pub case_set_sha256: String,
    pub executed_cases_claim: u64,
}

fn checked_hex(v: &str, width: usize) -> bool {
    v.len() == width && v.bytes().all(|b| b.is_ascii_hexdigit())
}
fn digest_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn verify_artifact_zip(
    zip_bytes: &[u8],
    github_digest_claim: &str,
    expected: &TrustedReceiptPolicy,
) -> Result<VerifiedArchive, String> {
    if zip_bytes.is_empty() || zip_bytes.len() > MAX_ARTIFACT_ZIP_BYTES {
        return Err("missing or oversized artifact archive".into());
    }
    if !github_digest_claim.starts_with("sha256:")
        || !checked_hex(&github_digest_claim[7..], 64)
    {
        return Err("invalid trusted GitHub artifact digest syntax".into());
    }
    if !checked_hex(&expected.source_sha, 40)
        || !checked_hex(&expected.tree_sha, 40)
        || !checked_hex(&expected.case_set_sha256, 64)
        || expected.run_id == 0 || expected.job_id == 0
    {
        return Err("invalid trusted fixture/run policy".into());
    }
    let archive_hash = digest_hex(zip_bytes);
    if !github_digest_claim[7..].eq_ignore_ascii_case(&archive_hash) {
        return Err("artifact ZIP bytes differ from GitHub SHA-256 claim".into());
    }
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|_| "artifact ZIP unreadable")?;
    if archive.len() != 2 {
        return Err("artifact requires exactly receipt and fixture-set files".into());
    }
    let mut receipt_bytes = None;
    let mut fixtures_bytes = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|_| "invalid archive entry")?;
        if !file.is_file() { return Err("artifact archive contains a directory".into()); }
        let (cap, place): (usize, &mut Option<Vec<u8>>) = match file.name() {
            "guest-receipt.json" => (MAX_RECEIPT_BYTES, &mut receipt_bytes),
            "fixture-set.json" => (MAX_FIXTURE_BYTES, &mut fixtures_bytes),
            _ => return Err("unexpected archive path (potential traversal)".into())
        };
        if place.is_some() { return Err("duplicate archive entry".into()); }
        if file.size() == 0 || file.size() > cap as u64 {
            return Err("missing or oversized decompressed file".into());
        }
        let mut decoded = Vec::new();
        (&mut file).take(cap as u64 + 1).read_to_end(&mut decoded)
            .map_err(|_| "ZIP CRC, compression, or I/O failure")?;
        if decoded.len() as u64 != file.size() || decoded.is_empty() || decoded.len() > cap {
            return Err("incomplete or oversized decompressed file".into());
        }
        *place = Some(decoded);
    }
    let fixture_set = fixtures_bytes.ok_or("missing fixture-set.json")?;
    let fixture_hash = digest_hex(&fixture_set);
    if !fixture_hash.eq_ignore_ascii_case(&expected.case_set_sha256) {
        return Err("actual fixture-set bytes do not match trusted SHA-256".into());
    }
    // Fixture-set is itself strict JSON, not a raw opaque digest string.
    if !decode_strict_json(&fixture_set)?.is_array() {
        return Err("fixture-set is not a JSON array".into());
    }
    let receipt_bytes = receipt_bytes.ok_or("missing guest-receipt.json")?;
    let receipt_value = decode_strict_json(&receipt_bytes)?;
    let r: Receipt = serde_json::from_value(receipt_value)
        .map_err(|_| "receipt schema invalid")?;
    if r.version != 1 || r.repository != expected.repository ||
        r.run_id != expected.run_id || r.job_id != expected.job_id ||
        r.source_sha != expected.source_sha || r.tree_sha != expected.tree_sha ||
        r.backend != expected.backend || r.feature_id != expected.feature_id ||
        !r.case_set_sha256.eq_ignore_ascii_case(&expected.case_set_sha256)
    {
        return Err("receipt does not match trusted run, tree, backend or fixtures".into());
    }
    if r.status != "passed" || r.execution_target != "guest" ||
        r.executed_cases == 0 || r.failed_cases != 0
    {
        return Err("unexecuted, host-only, failed or empty guest receipt".into());
    }
    Ok(VerifiedArchive {
        artifact_bytes_verified: true,
        fixture_bytes_verified: true,
        receipt_claim_consistent: true,
        guest_execution_verified: false,
        archive_sha256: archive_hash,
        case_set_sha256: fixture_hash,
        executed_cases_claim: r.executed_cases,
    })
}
