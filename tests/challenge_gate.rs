use atto_evidence_contract_conformance::{
    artifact_receipt::{ArtifactPolicy,CheckedGuestArtifact,GuestReceipt},
    challenge_gate::{ChallengeAuthority,ChallengeBinding,MAX_CHALLENGE_AGE_SECONDS},
};
use std::fs;
use std::sync::{Arc,Barrier};
use std::time::{SystemTime,UNIX_EPOCH};
const SOURCE:&str="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE:&str="cccccccccccccccccccccccccccccccccccccccc";
const CASES:&str="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const TIME:u64=1_804_700_000;
fn binding()->ChallengeBinding<'static>{
    ChallengeBinding{
        repository:"atto-lang/atto-source.llvm",source_sha:SOURCE,tree_sha:TREE,
        case_sha256:CASES,backend:"llvm",feature:"actor.m4.identical_counter",
        workflow:".github/workflows/m4-llvm-guest-receipt.yml",
        command:"bash ci/m4/verify-shared-counter.sh",
    }
}
struct PrivateDir(std::path::PathBuf);
impl PrivateDir {
    fn new()->Self{
        use std::sync::atomic::{AtomicU64,Ordering};
        static N:AtomicU64=AtomicU64::new(0);
        let ns=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path=std::env::temp_dir().join(format!(
            "atto-challenge-test-{}-{ns}-{}",std::process::id(),N.fetch_add(1,Ordering::Relaxed)));
        fs::create_dir(&path).unwrap();
        #[cfg(unix)]{
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();
        }
        Self(path)
    }
}
impl Drop for PrivateDir{
    fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}
}
fn materialize(nonce:String)->(ArtifactPolicy<'static>,CheckedGuestArtifact){
    let nonce_for_policy: &'static str=Box::leak(nonce.clone().into_boxed_str());
    let policy=ArtifactPolicy{
        repository:"atto-lang/atto-source.llvm",source_sha:SOURCE,tree_sha:TREE,
        case_sha256:CASES,backend:"llvm",feature:"actor.m4.identical_counter",
        run_id:42,run_attempt:1,job_id:123,artifact_id:8,
        artifact_digest:"sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        command:"bash ci/m4/verify-shared-counter.sh",nonce:nonce_for_policy,
        signer_workflow:".github/workflows/m4-llvm-guest-receipt.yml",
    };
    let receipt=GuestReceipt{
        version:1,repository:policy.repository.into(),source_sha:SOURCE.into(),
        tree_sha:TREE.into(),case_sha256:CASES.into(),backend:"llvm".into(),
        feature:policy.feature.into(),run_id:42,run_attempt:1,job_id:123,
        target:"guest".into(),command:policy.command.into(),nonce,
        executed:1,failed:0,exit_code:0,stdout_sha256:"a".repeat(64),
    };
    (policy,CheckedGuestArtifact{
        archive_sha256:"d".repeat(64),receipt,fixture_bytes_verified:false,
    })
}
#[test] fn issue_and_consume_once_and_reject_cross_process_replay(){
    let dir=PrivateDir::new();let a=ChallengeAuthority::open(&dir.0).unwrap();
    let token=a.issue(&binding(),TIME).unwrap();
    assert_eq!(token.expires_unix-TIME,MAX_CHALLENGE_AGE_SECONDS);
    assert_eq!(token.nonce.len(),64);
    let (policy,checked)=materialize(token.nonce);
    let reopened=ChallengeAuthority::open(&dir.0).unwrap();
    assert!(reopened.consume(&policy,&checked,TIME+10).is_ok());
    assert!(a.consume(&policy,&checked,TIME+11).is_err());
}
#[test] fn expired_future_and_wrong_binding_fail_closed(){
    let dir=PrivateDir::new();let a=ChallengeAuthority::open(&dir.0).unwrap();
    let c=a.issue(&binding(),TIME).unwrap();
    let (mut policy,mut receipt)=materialize(c.nonce);
    assert!(a.consume(&policy,&receipt,TIME-1).is_err());
    assert!(a.consume(&policy,&receipt,TIME+MAX_CHALLENGE_AGE_SECONDS).is_err());
    policy.tree_sha=SOURCE;
    assert!(a.consume(&policy,&receipt,TIME+10).is_err());
    policy.tree_sha=TREE;
    policy.backend="java";
    assert!(a.consume(&policy,&receipt,TIME+10).is_err());
    policy.backend="llvm";
    receipt.receipt.nonce="f".repeat(64);
    assert!(a.consume(&policy,&receipt,TIME+10).is_err());
    receipt.receipt.nonce=policy.nonce.into();
    assert!(a.consume(&policy,&receipt,TIME+10).is_ok());
}
#[test] fn self_issued_unregistered_nonce_or_tampered_issued_record_fail(){
    let dir=PrivateDir::new();let a=ChallengeAuthority::open(&dir.0).unwrap();
    let (policy,receipt)=materialize("e".repeat(64));
    assert!(a.consume(&policy,&receipt,TIME+1).is_err());
    let issued=a.issue(&binding(),TIME).unwrap();
    let path=dir.0.join(format!("{}.issued.json",
        atto_evidence_contract_conformance::artifact_receipt::sha256_hex(issued.nonce.as_bytes())));
    let source=fs::read_to_string(&path).unwrap();
    fs::write(&path,source.replace("\"version\":1","\"version\":2")).unwrap();
    let (policy,receipt)=materialize(issued.nonce);
    assert!(a.consume(&policy,&receipt,TIME+10).is_err());
}
#[test] fn concurrent_verification_accepts_exactly_one_nonce(){
    let dir=PrivateDir::new();let a=Arc::new(ChallengeAuthority::open(&dir.0).unwrap());
    let challenge=a.issue(&binding(),TIME).unwrap();
    let pair=Arc::new(materialize(challenge.nonce));
    let barrier=Arc::new(Barrier::new(8));
    let mut handles=Vec::new();
    for _ in 0..8 {
        let (a,pair,barrier)=(a.clone(),pair.clone(),barrier.clone());
        handles.push(std::thread::spawn(move||{
            barrier.wait();a.consume(&pair.0,&pair.1,TIME+1).is_ok()
        }));
    }
    assert_eq!(handles.into_iter().map(|t|t.join().unwrap()).filter(|ok|*ok).count(),1);
}
#[test] fn unsafe_journals_and_policy_are_rejected(){
    let dir=PrivateDir::new();
    let authority=ChallengeAuthority::open(&dir.0).unwrap();
    let bad=ChallengeBinding{backend:"LLVM",..binding()};
    assert!(authority.issue(&bad,TIME).is_err());
    let bad=ChallengeBinding{repository:"attacker/../escape",..binding()};
    assert!(authority.issue(&bad,TIME).is_err());
    #[cfg(unix)]{
        use std::os::unix::fs::{symlink,PermissionsExt};
        let link=dir.0.with_extension("symlink");
        symlink(&dir.0,&link).unwrap();
        assert!(ChallengeAuthority::open(&link).is_err());
        fs::remove_file(link).unwrap();
        fs::set_permissions(&dir.0,fs::Permissions::from_mode(0o755)).unwrap();
        assert!(ChallengeAuthority::open(&dir.0).is_err());
    }
}
