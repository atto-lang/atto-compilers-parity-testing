//! Independent verifier-issued, single-use M4/Atto guest challenges.
//!
//! The authority runs in a private, trusted verifier environment, never in
//! the untrusted guest-producing GitHub Actions job. It stores issued challenges
//! in an owner-private journal and atomically consumes each once after the
//! separately authenticated artifact and GitHub metadata have passed.
//! This protocol is not an artifact-signing service or proof of trusted runners.
use crate::artifact_receipt::{sha256_hex, ArtifactPolicy, CheckedGuestArtifact};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_CHALLENGE_AGE_SECONDS: u64 = 600;
const MAX_RECORD_BYTES: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssuedChallenge {
    pub version: u32,
    pub repository: String,
    pub source_sha: String,
    pub tree_sha: String,
    pub case_sha256: String,
    pub backend: String,
    pub feature: String,
    pub workflow: String,
    pub command: String,
    pub nonce: String,
    pub issued_unix: u64,
    pub expires_unix: u64,
}
#[derive(Clone, Debug)]
pub struct ChallengeBinding<'a> {
    pub repository: &'a str,
    pub source_sha: &'a str,
    pub tree_sha: &'a str,
    pub case_sha256: &'a str,
    pub backend: &'a str,
    pub feature: &'a str,
    pub workflow: &'a str,
    pub command: &'a str,
}
pub fn now_unix() -> Result<u64, String> {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs()).map_err(|_| "invalid system clock".into())
}
fn hex(s: &str, size: usize) -> bool {
    s.len()==size && s.bytes().all(|c|c.is_ascii_hexdigit()&&!c.is_ascii_uppercase())
}
fn valid_repository(s: &str) -> bool {
    let mut it=s.split('/');
    let (Some(owner), Some(repo), None)=(it.next(),it.next(),it.next()) else {return false};
    !owner.is_empty() && !repo.is_empty() && owner.len()<=39 && repo.len()<=100
      && !owner.starts_with('-') && !owner.ends_with('-')
      && repo!="." && repo!=".."
      && owner.bytes().all(|c| c.is_ascii_alphanumeric()||c==b'-')
      && repo.bytes().all(|c| c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_'|b'.'))
}
fn trusted_binding(b: &ChallengeBinding<'_>) -> bool {
    valid_repository(b.repository) && hex(b.source_sha,40) && hex(b.tree_sha,40)
      && hex(b.case_sha256,64) && !b.backend.is_empty() && b.backend.len()<=16
      && b.backend.bytes().all(|c| c.is_ascii_lowercase() || c==b'-')
      && !b.feature.is_empty() && b.feature.len()<=128
      && b.feature.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()
         || matches!(c,b'.'|b'_'|b'-'))
      && b.workflow.starts_with(".github/workflows/") && b.workflow.len()<=160
      && b.workflow.bytes().all(|c| c.is_ascii_alphanumeric()
         || matches!(c,b'/'|b'.'|b'_'|b'-'))
      && !b.command.is_empty() && b.command.len()<=1024
      && !b.command.contains('\n') && !b.command.contains('\r')
}
fn random_nonce() -> Result<String, String> {
    // Linux verifier host: OS CSPRNG; refuse weak fallback or an untrusted job RNG.
    #[cfg(unix)]
    {
        let mut f=File::open("/dev/urandom")
            .map_err(|_| "verifier OS CSPRNG is unavailable")?;
        let mut bytes=[0u8;32];
        f.read_exact(&mut bytes).map_err(|_| "verifier OS CSPRNG read failed")?;
        const HEX:&[u8;16]=b"0123456789abcdef";
        let mut result=String::with_capacity(64);
        for byte in bytes {
            result.push(HEX[(byte>>4) as usize] as char);
            result.push(HEX[(byte&15) as usize] as char);
        }
        Ok(result)
    }
    #[cfg(not(unix))]
    {Err("verifier-issued challenges require a supported CSPRNG platform".into())}
}
fn check_private_dir(path:&Path) -> Result<(),String> {
    let meta=fs::symlink_metadata(path).map_err(|_|"challenge authority directory missing")?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("challenge authority must be a real private directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err("challenge authority directory must be private (0700)".into());
        }
    }
    Ok(())
}
fn record_path(root:&Path,nonce:&str)->PathBuf {
    root.join(format!("{}.issued.json",sha256_hex(nonce.as_bytes())))
}
fn consumed_path(root:&Path,nonce:&str)->PathBuf {
    root.join(format!("{}.consumed",sha256_hex(nonce.as_bytes())))
}
fn new_private_file(path:&Path)->Result<File,String> {
    let mut options=OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(|_|"challenge already issued/consumed or storage inaccessible".into())
}
fn sync_parent(root:&Path)->Result<(),String> {
    File::open(root).and_then(|f|f.sync_all())
        .map_err(|_|"challenge journal directory sync failed".into())
}
pub struct ChallengeAuthority {root:PathBuf}
impl ChallengeAuthority {
    pub fn open(path:impl AsRef<Path>)->Result<Self,String>{
        let root=path.as_ref();
        check_private_dir(root)?;
        Ok(Self{root:root.to_path_buf()})
    }

    /// Only execute in a verifier-owned process. The resulting challenge nonce
    /// may be transmitted to the producer; the *journal* MUST remain private.
    pub fn issue(&self, binding:&ChallengeBinding<'_>, now:u64)
        ->Result<IssuedChallenge,String> {
        check_private_dir(&self.root)?;
        if !trusted_binding(binding) || now<1_700_000_000 {
            return Err("invalid independently selected challenge policy or clock".into());
        }
        let expires=now.checked_add(MAX_CHALLENGE_AGE_SECONDS)
            .ok_or("challenge expiration overflow")?;
        let nonce=random_nonce()?;
        let challenge=IssuedChallenge{
            version:1,repository:binding.repository.into(),source_sha:binding.source_sha.into(),
            tree_sha:binding.tree_sha.into(),case_sha256:binding.case_sha256.into(),
            backend:binding.backend.into(),feature:binding.feature.into(),
            workflow:binding.workflow.into(),command:binding.command.into(),
            nonce:nonce.clone(),issued_unix:now,expires_unix:expires,
        };
        let bytes=serde_json::to_vec(&challenge).map_err(|_|"cannot encode challenge")?;
        if bytes.len()>MAX_RECORD_BYTES{return Err("challenge record too large".into());}
        let mut file=new_private_file(&record_path(&self.root,&nonce))?;
        // A failed write or sync is NOT rolled back; no weak retry.
        file.write_all(&bytes).and_then(|_|file.sync_all())
            .map_err(|_|"failed to persist verifier-issued challenge")?;
        sync_parent(&self.root)?;
        Ok(challenge)
    }

    /// Execute ONLY after independent GitHub metadata, ZIP SHA and signature
    /// verification. Even this successful check is NOT guest certification.
    pub fn consume(
        &self, p:&ArtifactPolicy<'_>, checked:&CheckedGuestArtifact, now:u64
    )->Result<(),String>{
        check_private_dir(&self.root)?;
        let nonce=p.nonce;
        if !hex(nonce,64) || checked.receipt.nonce!=nonce {
            return Err("not an independently issued nonce".into());
        }
        let path=record_path(&self.root,nonce);
        let meta=fs::symlink_metadata(&path)
            .map_err(|_|"no independently issued challenge for guest nonce")?;
        if !meta.file_type().is_file() || meta.len()>MAX_RECORD_BYTES as u64 {
            return Err("challenge record must be a bounded regular file".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt,PermissionsExt};
            if meta.permissions().mode() & 0o077 != 0 || meta.nlink()!=1 {
                return Err("challenge record is not private/unique".into());
            }
        }
        let file=File::open(&path).map_err(|_|"cannot read issued challenge")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let opened=file.metadata().map_err(|_|"challenge record metadata error")?;
            if opened.ino()!=meta.ino() || opened.dev()!=meta.dev() {
                return Err("challenge record substituted during read".into());
            }
        }
        let mut bytes=Vec::new();
        file.take((MAX_RECORD_BYTES+1) as u64).read_to_end(&mut bytes)
            .map_err(|_|"challenge read failed")?;
        if bytes.len()>MAX_RECORD_BYTES{return Err("challenge exceeds size limit".into());}
        let json=crate::decode_strict_json(&bytes)?;
        let c:IssuedChallenge=serde_json::from_value(json)
            .map_err(|_|"challenge fields invalid")?;
        if c.version!=1 || c.nonce!=nonce
            || c.issued_unix<1_700_000_000 || c.expires_unix<=c.issued_unix
            || c.expires_unix-c.issued_unix>MAX_CHALLENGE_AGE_SECONDS
            || now<c.issued_unix || now>=c.expires_unix {
            return Err("challenge expired, malformed, or issued in the future".into());
        }
        let r=&checked.receipt;
        if c.repository!=p.repository || c.source_sha!=p.source_sha
            || c.tree_sha!=p.tree_sha || c.case_sha256!=p.case_sha256
            || c.backend!=p.backend || c.feature!=p.feature
            || c.workflow!=p.signer_workflow || c.command!=p.command
            || r.repository!=c.repository || r.source_sha!=c.source_sha
            || r.tree_sha!=c.tree_sha || r.case_sha256!=c.case_sha256
            || r.backend!=c.backend || r.feature!=c.feature
            || r.command!=c.command || r.nonce!=c.nonce {
            return Err("challenge not bound to verified source, fixture and command".into());
        }
        let marker=consumed_path(&self.root,nonce);
        let mut out=new_private_file(&marker)?;
        out.write_all(format!("{}|{}|{}|{}|{}\n",
            r.repository,r.run_id,r.run_attempt,r.job_id,r.source_sha).as_bytes())
            .and_then(|_|out.sync_all())
            .map_err(|_|"failed to persist consumed challenge")?;
        sync_parent(&self.root)?;
        Ok(())
    }
}
