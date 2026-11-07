//! Disabled-by-default private detached-signature verification.
//! No private key, signing service or KMS credentials are supplied here.
//! Signers must be independently governed and must verify the execution
//! before signing; a valid signature alone does NOT certify guest execution.
use crate::artifact_receipt::sha256_hex;
use std::{fs::{self,File,OpenOptions},io::Write,path::{Path,PathBuf},
    process::{Command,Stdio},sync::atomic::{AtomicU64,Ordering}};
const MAX_KEY:usize=8192;
const MAX_STATEMENT:usize=4096;
const SIG_BYTES:usize=64;
static NEXT:AtomicU64=AtomicU64::new(0);

pub struct ApprovedKey<'a>{
    pub key_id:&'a str,
    pub pem:&'a [u8],
    /// SHA256 of the EXACT reviewed public key bytes, configured out of band.
    pub pinned_sha256:&'a str,
    pub valid_from:u64,
    pub valid_until:u64,
    pub revoked:bool,
}
#[derive(Debug,Clone)]
pub struct SigningContext<'a>{
    pub repository:&'a str, pub source_sha:&'a str,
    pub tree_sha:&'a str,pub case_sha256:&'a str,
    pub run_id:u64,pub run_attempt:u64,pub job_id:u64,
    pub artifact_sha256:&'a str,
    pub challenge_nonce:&'a str,
    pub signer_workflow:&'a str,
    pub runner_identity:&'a str,
    pub backend:&'a str,
    pub feature:&'a str,
}
fn hex(s:&str,n:usize)->bool{
    s.len()==n&&s.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())
}
fn atom(s:&str)->bool{
    !s.is_empty()&&s.len()<=192&&s.bytes().all(|b|
        b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_'|b'.'|b'/'|b':'))
}
fn repo(s:&str)->bool{
    let mut p=s.split('/');
    let (Some(a),Some(b),None)=(p.next(),p.next(),p.next())else{return false};
    atom(a)&&atom(b)&&a.len()<=39&&b.len()<=100
        &&a!="."&&a!=".."&&b!="."&&b!=".."
}
/// Fixed-order, domain-separated bytes. A signature MUST cover this entire
/// context; a detached signature over the ZIP alone cannot prove its source.
pub fn statement(c:&SigningContext<'_>)->Result<Vec<u8>,String>{
    if !repo(c.repository)||!hex(c.source_sha,40)||!hex(c.tree_sha,40)
        ||!hex(c.case_sha256,64)||!hex(c.artifact_sha256,64)
        ||!hex(c.challenge_nonce,64)||c.run_id==0||c.run_attempt==0||c.job_id==0
        ||!c.signer_workflow.starts_with(".github/workflows/")
        ||![c.signer_workflow,c.runner_identity,c.backend,c.feature].iter().all(|s|atom(s)){
        return Err("invalid bound signing statement".into())
    }
    let s=format!(concat!(
        "atto-private-guest-attestation-v1\n",
        "repository={}\nsource_sha={}\ntree_sha={}\n",
        "case_sha256={}\nrun_id={}\nrun_attempt={}\njob_id={}\n",
        "artifact_sha256={}\nchallenge_nonce={}\n",
        "signer_workflow={}\nrunner_identity={}\nbackend={}\nfeature={}\n"
    ),c.repository,c.source_sha,c.tree_sha,c.case_sha256,
        c.run_id,c.run_attempt,c.job_id,c.artifact_sha256,c.challenge_nonce,
        c.signer_workflow,c.runner_identity,c.backend,c.feature);
    if s.len()>MAX_STATEMENT {return Err("statement oversized".into())}
    Ok(s.into_bytes())
}
pub trait PrivateSigningService{
    /// KMS/HSM integration intentionally NOT implemented in this repository.
    /// A service MUST verify source/runner provenance BEFORE signing.
    fn sign_checked_statement(&self,bytes:&[u8])->Result<Vec<u8>,String>;
}
/// The production default refuses any signing operation.
pub struct SigningDisabled;
impl PrivateSigningService for SigningDisabled{
    fn sign_checked_statement(&self,_:&[u8])->Result<Vec<u8>,String>{
        Err("private signing provider is not approved/configured".into())
    }
}
struct TempFile(PathBuf);
impl Drop for TempFile{
    fn drop(&mut self){let _=fs::remove_file(&self.0);}
}
fn temp(bytes:&[u8])->Result<TempFile,String>{
    for _ in 0..32{
        let path=std::env::temp_dir().join(format!("atto-detached-{}-{}",
            std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        let mut options=OpenOptions::new(); options.create_new(true).write(true);
        #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
        match options.open(&path){
            Ok(mut f)=>{
                let guard=TempFile(path);
                f.write_all(bytes).and_then(|_|f.sync_all())
                    .map_err(|_|"private verifier file write failed")?;
                return Ok(guard)
            }
            Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,
            Err(_)=>return Err("private verifier file allocation failed".into())
        }
    }
    Err("private verifier file allocation exhausted".into())
}
fn approved_key(k:&ApprovedKey<'_>,now:u64)->Result<(),String>{
    if !atom(k.key_id)||k.pem.is_empty()||k.pem.len()>MAX_KEY
        ||!k.pem.starts_with(b"-----BEGIN PUBLIC KEY-----\n")
        ||!k.pem.ends_with(b"-----END PUBLIC KEY-----\n")
        ||!hex(k.pinned_sha256,64)||sha256_hex(k.pem)!=k.pinned_sha256
        ||k.valid_from==0||k.valid_until<=k.valid_from
        ||now<k.valid_from||now>=k.valid_until||k.revoked{
        return Err("approved key pin, validity or revocation check failed".into())
    }
    Ok(())
}
/// Verify a 64-byte Ed25519 signature using the exact approved and pinned
/// public key. Never reads a private key and never calls a network endpoint.
/// A cryptographically verified context statement is *not* proof of a trusted
/// runner; the signing authority's policy must be audited separately.
pub fn verify_detached(c:&SigningContext<'_>,sig:&[u8],
    key:&ApprovedKey<'_>,now:u64)->Result<(),String>{
    approved_key(key,now)?;
    if sig.len()!=SIG_BYTES {return Err("Ed25519 signature must be exactly 64 bytes".into())}
    let bytes=statement(c)?;
    let public=temp(key.pem)?;
    let message=temp(&bytes)?;
    let signature=temp(sig)?;
    let status=Command::new("openssl")
        .args(["pkeyutl","-verify","-rawin","-pubin","-inkey"])
        .arg(&public.0).arg("-in").arg(&message.0)
        .arg("-sigfile").arg(&signature.0)
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .status().map_err(|_|"OpenSSL Ed25519 verification unavailable")?;
    if !status.success(){return Err("private attestation signature verification failed".into())}
    Ok(())
}
