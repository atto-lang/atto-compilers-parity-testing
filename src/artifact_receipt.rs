//! Strict archive-byte checks for independently supplied guest-execution evidence.
//! A GitHub SHA-256 digest alone is not signed provenance or proof of guest execution.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet,io::{Cursor,Read,Write},process::{Command,Stdio}};
use zip::{ZipArchive,CompressionMethod};

pub const MAX_ARCHIVE_BYTES:usize=16*1024*1024;
#[derive(Debug,Clone)]
pub struct ArtifactPolicy<'a>{
    pub repository:&'a str, pub source_sha:&'a str, pub tree_sha:&'a str,
    pub case_sha256:&'a str, pub backend:&'a str, pub feature:&'a str,
    pub run_id:u64, pub run_attempt:u64, pub job_id:u64, pub artifact_id:u64,
    pub artifact_digest:&'a str, pub command:&'a str, pub nonce:&'a str,
    pub signer_workflow:&'a str,
}
#[derive(Debug,Clone,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestReceipt{
    pub version:u32, pub repository:String, pub source_sha:String,
    pub tree_sha:String, pub case_sha256:String, pub backend:String,
    pub feature:String, pub run_id:u64, pub run_attempt:u64, pub job_id:u64,
    pub target:String, pub command:String, pub nonce:String,
    pub executed:u64, pub failed:u64, pub exit_code:i32,
    pub stdout_sha256:String,
}
#[derive(Debug,Clone)]
pub struct CheckedGuestArtifact{
    pub archive_sha256:String,
    pub receipt:GuestReceipt,
    /// True only if the artifact carries the exact bytes of the trusted
    /// fixture-set.json and its case count agrees with the guest receipt.
    pub fixture_bytes_verified:bool,
}
pub fn sha256_hex(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
fn hex(s:&str,n:usize)->bool{
    s.len()==n && s.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())
}
fn safe_repo(s:&str)->bool{
    let mut it=s.split('/');
    let (Some(a),Some(b),None)=(it.next(),it.next(),it.next())else{return false};
    !a.is_empty()&&!b.is_empty()&&a.len()<=39&&b.len()<=100
    &&!a.starts_with('-')&&!a.ends_with('-')&&b!="."&&b!=".."
    &&a.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'-')
    &&b.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_'|b'.'))
}
fn policy_valid(p:&ArtifactPolicy<'_>)->bool{
    safe_repo(p.repository)&&hex(p.source_sha,40)&&hex(p.tree_sha,40)
    &&hex(p.case_sha256,64)&&p.run_id>0&&p.run_attempt>0&&p.job_id>0
    &&p.artifact_id>0&&p.artifact_digest.starts_with("sha256:")
    &&hex(&p.artifact_digest[7..],64)
    &&!p.command.is_empty()&&p.command.len()<=1024
    &&!p.backend.is_empty()&&p.backend.len()<=16
    &&!p.feature.is_empty()&&p.feature.len()<=128
    &&p.nonce.len()>=32&&p.nonce.len()<=128
    &&p.nonce.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-')
    &&p.signer_workflow.starts_with(".github/workflows/")
    &&p.signer_workflow.len()<=160
    &&p.signer_workflow.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_'|b'.'|b'/'))
}
pub fn validate_archive(bytes:&[u8],p:&ArtifactPolicy<'_>)->Result<CheckedGuestArtifact,String>{
    if !policy_valid(p){return Err("invalid trusted policy".into());}
    if bytes.is_empty()||bytes.len()>MAX_ARCHIVE_BYTES{return Err("archive absent or oversized".into());}
    let digest=sha256_hex(bytes);
    if p.artifact_digest!=format!("sha256:{digest}"){
        return Err("actual ZIP SHA-256 does not match trusted GitHub artifact digest".into());
    }
    let mut zip=ZipArchive::new(Cursor::new(bytes)).map_err(|e|e.to_string())?;
    if zip.len()!=2 && zip.len()!=3 {
        return Err("guest ZIP requires receipt, stdout and optional fixture set".into());
    }
    let mut receipt=None;
    let mut output=None;
    let mut fixture_set=None;
    let mut seen=BTreeSet::new();
    for i in 0..zip.len(){
        let mut f=zip.by_index(i).map_err(|e|e.to_string())?;
        let name=f.name().to_owned();
        if !seen.insert(name.clone())||f.is_dir()
           ||name!="guest-receipt.json"&&name!="guest-stdout.bin"&&name!="fixture-set.json"{
            return Err("unexpected or duplicate ZIP path".into());
        }
        if f.encrypted()||!matches!(f.compression(),CompressionMethod::Stored|CompressionMethod::Deflated){
            return Err("encrypted or unsupported ZIP member".into());
        }
        if let Some(mode)=f.unix_mode() {
            let kind=mode & 0o170000;
            if kind!=0 && kind!=0o100000 {
                return Err("special files and ZIP symlinks are forbidden".into());
            }
        }
        let bound=match name.as_str() {
            "guest-receipt.json"=>32768,
            "fixture-set.json"=>4*1024*1024,
            _=>1048576,
        };
        if f.size()>bound||f.compressed_size()==0&&f.size()>0
            ||f.compressed_size()>0&&f.size()>f.compressed_size().saturating_mul(500){
            return Err("oversized or suspicious compression ratio".into());
        }
        let expected_size=f.size();
        let mut buf=Vec::new();
        f.take(bound+1).read_to_end(&mut buf).map_err(|e|e.to_string())?;
        if buf.len()>bound as usize || buf.len() as u64!=expected_size {
            return Err("ZIP inflated past cap or was truncated".into());
        }
        match name.as_str() {
            "guest-receipt.json"=>receipt=Some(buf),
            "fixture-set.json"=>fixture_set=Some(buf),
            _=>output=Some(buf),
        }
    }
    let raw=receipt.ok_or("missing receipt")?;
    let stdout=output.ok_or("missing captured output")?;
    // serde_json::from_slice alone would silently accept duplicate property names.
    let parsed=crate::decode_strict_json(&raw)?;
    let r:GuestReceipt=serde_json::from_value(parsed).map_err(|e|e.to_string())?;
    if r.version!=1||r.repository!=p.repository||r.source_sha!=p.source_sha
        ||r.tree_sha!=p.tree_sha||r.case_sha256!=p.case_sha256
        ||r.backend!=p.backend||r.feature!=p.feature||r.run_id!=p.run_id
        ||r.run_attempt!=p.run_attempt||r.job_id!=p.job_id||r.target!="guest"
        ||r.command!=p.command||r.nonce!=p.nonce||r.executed==0||r.failed!=0
        ||r.exit_code!=0||!hex(&r.stdout_sha256,64)||r.stdout_sha256!=sha256_hex(&stdout){
        return Err("guest receipt does not match exact run, tree, nonce or output".into());
    }
    let fixture_bytes_verified=if let Some(fixtures)=fixture_set {
        if sha256_hex(&fixtures)!=p.case_sha256 {
            return Err("embedded fixture-set bytes do not match trusted SHA-256".into());
        }
        let parsed=crate::decode_strict_json(&fixtures)?;
        let cases=parsed.as_array().ok_or("fixture-set.json must be a JSON array")?;
        if cases.is_empty() || cases.len() as u64!=r.executed {
            return Err("claimed executed cases do not match trusted fixture-set count".into());
        }
        true
    } else { false };
    Ok(CheckedGuestArtifact{
        archive_sha256:digest,receipt:r,fixture_bytes_verified,
    })
}
pub trait ProvenanceVerifier{
    fn verify_signed(&self,bytes:&[u8],p:&ArtifactPolicy<'_>)->Result<(),String>;
}
pub fn validate_signed<V:ProvenanceVerifier>(
    bytes:&[u8],p:&ArtifactPolicy<'_>,verifier:&V
)->Result<CheckedGuestArtifact,String>{
    let checked=validate_archive(bytes,p)?;
    verifier.verify_signed(bytes,p)?;
    Ok(checked)
}
#[derive(Default)]
pub struct ReplayFence{seen:BTreeSet<String>}
impl ReplayFence{
    pub fn admit(&mut self,verified:&CheckedGuestArtifact)->Result<(),String>{
        let r=&verified.receipt;
        let id=format!("{}:{}:{}:{}:{}:{}:{}:{}",r.repository,r.source_sha,r.run_id,
            r.run_attempt,r.job_id,r.backend,r.feature,r.nonce);
        if !self.seen.insert(id){return Err("duplicate/replayed receipt in batch".into());}
        Ok(())
    }
}
fn bounded(mut command:Command,max:usize)->Result<Vec<u8>,String>{
    let mut child=command.stdin(Stdio::null()).stderr(Stdio::null())
        .stdout(Stdio::piped()).spawn().map_err(|e|e.to_string())?;
    let mut buf=Vec::new();
    let read=child.stdout.take().ok_or("missing command stdout")?
        .take(max as u64+1).read_to_end(&mut buf);
    if read.is_err()||buf.len()>max{
        let _=child.kill();let _=child.wait();
        return Err("untrusted command output failed or exceeded cap".into());
    }
    let status=child.wait().map_err(|e|e.to_string())?;
    if !status.success(){return Err(format!("GitHub read-only CLI failed: {status}"));}
    Ok(buf)
}
pub fn fetch_zip(p:&ArtifactPolicy<'_>)->Result<Vec<u8>,String>{
    if !policy_valid(p){return Err("unsafe API selectors".into());}
    let mut command=Command::new("gh");
    command.args(["api","--hostname","github.com","-X","GET"])
        .arg(format!("/repos/{}/actions/artifacts/{}/zip",p.repository,p.artifact_id))
        .env("GH_PROMPT_DISABLED","1");
    bounded(command,MAX_ARCHIVE_BYTES)
}
pub struct GhProvenance;
impl ProvenanceVerifier for GhProvenance{
    fn verify_signed(&self,bytes:&[u8],p:&ArtifactPolicy<'_>)->Result<(),String>{
        if !policy_valid(p){return Err("unsafe attestation selectors".into());}
        // Avoid arbitrary shell pipelines and untrusted report-provided paths.
        use std::sync::atomic::{AtomicU64,Ordering};
        static NEXT:AtomicU64=AtomicU64::new(0);
        let mut tmp=None;
        for _ in 0..16{
            let path=std::env::temp_dir().join(format!("atto-guest-{}-{}.zip",
                std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
            let mut options=std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {use std::os::unix::fs::OpenOptionsExt; options.mode(0o600);}
            match options.open(&path){
                Ok(mut file)=>{
                    if let Err(e)=file.write_all(bytes){let _=std::fs::remove_file(&path);return Err(e.to_string())}
                    tmp=Some(path);break;
                }
                Err(e)if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,
                Err(e)=>return Err(e.to_string()),
            }
        }
        let path=tmp.ok_or("cannot create unique archive for attestation")?;
        let signer=format!("github.com/{}/{}",p.repository,p.signer_workflow);
        let mut command=Command::new("gh");
        command.args(["attestation","verify"]).arg(&path)
            .args(["--hostname","github.com","--repo",p.repository,
                   "--source-digest",p.source_sha,"--signer-workflow",&signer,
                   "--format","json"])
            .env("GH_PROMPT_DISABLED","1");
        let out=bounded(command,2*1024*1024);
        let _=std::fs::remove_file(path);
        let proof=crate::decode_strict_json(&out?)?;
        let entries=proof.as_array().ok_or("no signed attestations")?;
        let digest=sha256_hex(bytes);
        let verified=entries.iter().any(|item|{
            let v=&item["verificationResult"];
            v["statement"]["predicateType"].as_str()==Some("https://slsa.dev/provenance/v1")
                &&v["signature"]["certificate"].is_object()
                &&v["verifiedTimestamps"].as_array().map(|x|!x.is_empty()).unwrap_or(false)
                &&v["statement"]["subject"].as_array().map(|arr|arr.iter().any(|s|
                    s["digest"]["sha256"].as_str()==Some(digest.as_str())
                )).unwrap_or(false)
        });
        if !verified{return Err("missing cryptographically verified artifact provenance".into());}
        Ok(())
    }
}
pub fn fetch_and_verify(p:&ArtifactPolicy<'_>)->Result<CheckedGuestArtifact,String>{
    let bytes=fetch_zip(p)?;
    validate_signed(&bytes,p,&GhProvenance)
}
