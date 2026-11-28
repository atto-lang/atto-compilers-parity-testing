//! Issue a challenge on an INDEPENDENT trusted verifier host only.
//! This tool never signs artifacts or authenticates an unreviewed producer.
use atto_evidence_contract_conformance::challenge_gate::{
    now_unix,ChallengeAuthority,ChallengeBinding
};
fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args.len()!=9 {
        eprintln!("usage: challenge_issue REPO SOURCE_SHA TREE_SHA CASE_SHA256 BACKEND FEATURE WORKFLOW COMMAND");
        std::process::exit(2);
    }
    let result=(||->Result<String,String>{
        let dir=std::env::var("ATTO_CHALLENGE_LEDGER_DIR")
            .map_err(|_|"ATTO_CHALLENGE_LEDGER_DIR must be a trusted owner-private path")?;
        let b=ChallengeBinding{
            repository:&args[1],source_sha:&args[2],tree_sha:&args[3],
            case_sha256:&args[4],backend:&args[5],feature:&args[6],
            workflow:&args[7],command:&args[8],
        };
        let record=ChallengeAuthority::open(dir)?.issue(&b,now_unix()?)?;
        serde_json::to_string_pretty(&record).map_err(|e|e.to_string())
    })();
    match result{
        Ok(challenge)=>{
            println!("{challenge}");
            eprintln!("Issued a 10-minute challenge. Transfer nonce to producer securely.");
            eprintln!("The private challenge ledger must stay on the verifier host.");
        },
        Err(e)=>{eprintln!("NOT_CERTIFIED: {e}");std::process::exit(2)}
    }
}
