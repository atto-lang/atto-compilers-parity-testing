namespace Atto.Conformance.V1;

alias Backend = "java" | "llvm" | "wasm" | "forge" | "net";
alias ExecutionTarget = "host" | "guest";

model EvidenceBase {
  version: 1;
  backend: Backend;
  @pattern("^[a-z][a-z0-9_.-]{0,127}$")
  feature_id: string;
  @pattern("^[a-f0-9]{40}$")
  source_sha: string;
  @pattern("^[a-f0-9]{64}$")
  case_set_sha256: string;
}

model PassedEvidence extends EvidenceBase {
  status: "passed";
  execution_target: ExecutionTarget;
  run_url: url;
  @minValue(1)
  executed_cases: uint32;
  failed_cases: 0;
}

model FailedEvidence extends EvidenceBase {
  status: "failed";
  execution_target: ExecutionTarget;
  run_url: url;
  @minValue(1)
  executed_cases: uint32;
  @minValue(1)
  failed_cases: uint32;
  @minLength(1)
  @maxLength(512)
  reason: string;
}

model UnsupportedEvidence extends EvidenceBase {
  status: "unsupported";
  @minLength(1)
  @maxLength(512)
  reason: string;
}

model NotExecutedEvidence extends EvidenceBase {
  status: "not_executed";
  @minLength(1)
  @maxLength(512)
  reason: string;
}

union ConformanceEvidence {
  passed: PassedEvidence,
  failed: FailedEvidence,
  unsupported: UnsupportedEvidence,
  not_executed: NotExecutedEvidence,
}
