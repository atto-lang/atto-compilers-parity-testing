use atto_evidence_contract_conformance::{
    private_signature::{ApprovedKey,SigningContext,PrivateSigningService,SigningDisabled,
        statement,verify_detached},
    artifact_receipt::sha256_hex,
};
use std::{fs,process::Command,time::{SystemTime,UNIX_EPOCH}};
const S:&str="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const T:&str="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C:&str="cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const A:&str="dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const N:&str="eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const TIME:u64=1_804_720_000;
fn context()->SigningContext<'static>{SigningContext{
    repository:"atto-lang/atto-source.llvm",source_sha:S,tree_sha:T,
    case_sha256:C,artifact_sha256:A,challenge_nonce:N,run_id:42,run_attempt:1,
    job_id:9,signer_workflow:".github/workflows/m4-llvm-guest-receipt.yml",
    runner_identity:"trusted-runner-pool:linux-llvm-18",
    backend:"llvm",feature:"actor.m4.identical_counter"
}}
struct Workspace(std::path::PathBuf);
impl Workspace{
    fn new()->Self{
        let ts=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let p=std::env::temp_dir().join(format!("atto-key-test-{}-{ts}",std::process::id()));
        fs::create_dir(&p).unwrap();
        #[cfg(unix)]{
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&p,fs::Permissions::from_mode(0o700)).unwrap();
        }
        Self(p)
    }
    fn generate(&self,name:&str)->(Vec<u8>,Vec<u8>){
        let secret=self.0.join(format!("{name}-private.pem"));
        let public=self.0.join(format!("{name}-public.pem"));
        assert!(Command::new("openssl").args(["genpkey","-algorithm","ED25519","-out"])
            .arg(&secret).status().unwrap().success());
        assert!(Command::new("openssl").args(["pkey","-in"]).arg(&secret)
            .args(["-pubout","-out"]).arg(&public).status().unwrap().success());
        let message=self.0.join(format!("{name}-message"));
        let signature=self.0.join(format!("{name}-signature"));
        fs::write(&message,statement(&context()).unwrap()).unwrap();
        assert!(Command::new("openssl").args(["pkeyutl","-sign","-rawin","-inkey"])
            .arg(&secret).arg("-in").arg(&message).arg("-out")
            .arg(&signature).status().unwrap().success());
        (fs::read(&public).unwrap(),fs::read(&signature).unwrap())
    }
}
impl Drop for Workspace{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn key<'a>(pem:&'a [u8],sha:&'a str)->ApprovedKey<'a>{
    ApprovedKey{key_id:"KMS-PROVIDER-TEST-ONLY",pem,pinned_sha256:sha,
        valid_from:TIME-100,valid_until:TIME+100,revoked:false}
}
#[test]fn valid_ed25519_signature_covers_entire_exact_execution_context(){
    let w=Workspace::new();let(pem,sig)=w.generate("real");
    let pin=sha256_hex(&pem);verify_detached(&context(),&sig,&key(&pem,&pin),TIME).unwrap();
    for context in [
        SigningContext{job_id:10,..context()},
        SigningContext{run_attempt:2,..context()},
        SigningContext{artifact_sha256:C,..context()},
        SigningContext{source_sha:T,..context()},
        SigningContext{challenge_nonce:A,..context()},
        SigningContext{runner_identity:"unknown-runner",..context()},
        SigningContext{case_sha256:A,..context()},
        SigningContext{signer_workflow:".github/workflows/evil.yml",..context()},
    ] {
        assert!(verify_detached(&context,&sig,&key(&pem,&pin),TIME).is_err());
    }
}
#[test]fn wrong_key_pin_revocation_expiry_missing_or_malformed_signature_are_denied(){
    let w=Workspace::new();let(pem,mut sig)=w.generate("original");
    let(other_pem,_)=w.generate("different");
    let pin=sha256_hex(&pem);
    assert!(verify_detached(&context(),&sig,&key(&other_pem,&pin),TIME).is_err());
    assert!(verify_detached(&context(),&sig,&key(&pem,&"f".repeat(64)),TIME).is_err());
    assert!(verify_detached(&context(),&sig,&key(&pem,&pin),TIME+100).is_err());
    assert!(verify_detached(&context(),&sig,&key(&pem,&pin),TIME-101).is_err());
    let revoked=ApprovedKey{revoked:true,..key(&pem,&pin)};
    assert!(verify_detached(&context(),&sig,&revoked,TIME).is_err());
    sig[0]^=0x80;
    assert!(verify_detached(&context(),&sig,&key(&pem,&pin),TIME).is_err());
    assert!(verify_detached(&context(),&[],&key(&pem,&pin),TIME).is_err());
}
#[test]fn unsigned_default_signing_is_disabled_and_malicious_context_rejected(){
    assert!(SigningDisabled.sign_checked_statement(b"arbitrary evidence").is_err());
    assert!(statement(&SigningContext{repository:"owner/repo\nsigner=evil",..context()}).is_err());
    assert!(statement(&SigningContext{backend:"llvm\nrelease=yes",..context()}).is_err());
    assert!(statement(&SigningContext{run_id:0,..context()}).is_err());
}
