//! Read-only GitHub metadata probe: gh must already be authenticated.
//! This prints NOT_CERTIFIED even for perfect GitHub metadata: it cannot
//! independently attest ZIP bytes or guest runtime receipts.
use atto_evidence_contract_conformance::github_probe::{GhCli,ProbeSpec,probe_github_metadata};
fn main(){
    let a:Vec<_>=std::env::args().collect();
    if a.len()!=12 {
        eprintln!("usage: github_metadata_probe OWNER/REPO PR RUN SOURCE_SHA TREE_SHA WORKFLOW_PATH JOB_NAME REVIEWER ARTIFACT_NAME STEP_NAME STEP_NAME");
        std::process::exit(2);
    }
    let pr=a[2].parse().unwrap_or(0);
    let run=a[3].parse().unwrap_or(0);
    let required_steps=[a[10].as_str(),a[11].as_str()];
    let spec=ProbeSpec{
        repository:&a[1],pr_number:pr,run_id:run,
        source_sha:&a[4],source_tree_sha:&a[5],
        workflow_path:&a[6],job_name:&a[7],
        approved_reviewer:&a[8],artifact_name:&a[9],
        required_steps:&required_steps,
    };
    match probe_github_metadata(&GhCli,&spec){
        Ok(s)=>{
            println!("{}",serde_json::to_string_pretty(&s).expect("serialize"));
            // A metadata snapshot is NEVER a release certificate.
            std::process::exit(3);
        }
        Err(err)=>{
            eprintln!("NOT_CERTIFIED: {err}");
            std::process::exit(2);
        }
    }
}
