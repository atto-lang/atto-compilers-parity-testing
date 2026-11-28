//! Offline private Ed25519 *claim* verifier. This is NOT a release gate.
//! Approved key PIN, dates, revocation status and execution context MUST
//! originate from independent reviewed configuration, not from an artifact.
//! No signing keys, KMS access or public transparency log are used.
use atto_evidence_contract_conformance::{
    artifact_receipt::{sha256_hex,MAX_ARCHIVE_BYTES},
    private_signature::{ApprovedKey,SigningContext,verify_detached},
};
use std::{env,fs,time::{SystemTime,UNIX_EPOCH}};
fn read_bounded(path:&str,cap:u64)->Result<Vec<u8>,String>{
    let m=fs::symlink_metadata(path).map_err(|_|"evidence file missing")?;
    if !m.file_type().is_file()||m.len()==0||m.len()>cap{
        return Err("evidence file absent, nonregular, or over size cap".into())
    }
    fs::read(path).map_err(|_|"cannot read evidence input".into())
}
fn parse_positive(s:&str)->Result<u64,String>{
    s.parse::<u64>().ok().filter(|n|*n>0).ok_or("invalid positive number".into())
}
fn main(){
    let a:Vec<String>=env::args().collect();
    if a.len()!=21 {
        eprintln!("usage: private_signature_verify PUBLIC_PEM SIG64 ARTIFACT_ZIP APPROVED_KEY_SHA256 KEY_ID VALID_FROM VALID_UNTIL REVOKED(false) REPO HEAD TREE CASE_SHA256 RUN_ID ATTEMPT JOB_ID NONCE SIGNER_WORKFLOW RUNNER_ID BACKEND FEATURE");
        std::process::exit(2)
    }
    let result=(||->Result<(),String>{
        if a[8]!="false"{
            return Err("revoked, unknown or unapproved signer status".into())
        }
        let public=read_bounded(&a[1],8192)?;
        let sig=read_bounded(&a[2],64)?;
        let zip=read_bounded(&a[3],MAX_ARCHIVE_BYTES as u64)?;
        let digest=sha256_hex(&zip);
        let key=ApprovedKey{
            key_id:&a[5],pem:&public,pinned_sha256:&a[4],
            valid_from:parse_positive(&a[6])?,
            valid_until:parse_positive(&a[7])?,
            revoked:false,
        };
        let claim=SigningContext{
            repository:&a[9],source_sha:&a[10],tree_sha:&a[11],
            case_sha256:&a[12],run_id:parse_positive(&a[13])?,
            run_attempt:parse_positive(&a[14])?,job_id:parse_positive(&a[15])?,
            artifact_sha256:&digest,challenge_nonce:&a[16],
            signer_workflow:&a[17],runner_identity:&a[18],
            backend:&a[19],feature:&a[20],
        };
        let now=SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|_|"invalid trusted verifier clock")?.as_secs();
        verify_detached(&claim,&sig,&key,now)?;
        println!("{{\"status\":\"NOT_CERTIFIED\",\"detached_signature\":\"ED25519_VERIFIED\",\"archive_sha256\":\"{digest}\",\"runner_identity\":\"CLAIM_ONLY_NOT_INDEPENDENTLY_VERIFIED\"}}");
        Ok(())
    })();
    match result{
        Ok(())=>std::process::exit(3),
        Err(e)=>{eprintln!("NOT_CERTIFIED: {e}");std::process::exit(2)}
    }
}
