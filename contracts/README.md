# Cross-backend conformance-evidence contracts — draft v1

JSON Schema 2020-12 is normative for JSON report ingress. TypeSpec models
tagged status variants; WIT offers a proposed component-model reporter.
These files are not evidence that any compiler or runtime implements the
contract, and a valid JSON report is never independent proof of an executed test.

## Rust-first verification

Run cargo +1.88.0 test --all-targets for the strict Rust ingress decoder,
validation, malformed-input and negative-vector tests, status/field projections,
and cross-field case-count consistency. CI additionally runs the TypeSpec
compiler and a WIT parser.

The Rust ingress rejects duplicate JSON object keys, nonintegral/nonfinite
numeric tokens, malformed UTF-8/Unicode, and messages above 1 MiB before
JSON Schema validation. A separate semantic check rejects failed_cases
greater than executed_cases. The Rust tests cover valid shape and regression
cases; they do not contact GitHub, run a guest compiler, or certify a release.

## Evidence acceptance — fail closed

A passed or failed report is only a claim. An actual release or parity
attestor must independently verify the GitHub run URL's expected trusted
repository and run ID, latest exact source SHA/tree, resolved workflow status,
executed job steps, artifact provenance and digest, and fixture-set hash.
It must reject stale, skipped, pending, forged, or host-only evidence when
guest conformance is required. Do not use an untrusted report URL as a generic
network fetch target. Absent or unsupported is never equivalent to passed.

Four distinct statuses must remain intact: passed, failed, unsupported,
and not_executed. Each backend needs its own verified guest evidence;
a contract-only CI pass does not prove Java/GraalVM, LLVM, Wasm, Forge,
or .NET execution. Breaking changes require versioned fixtures and
cross-backend consumer verification.

All CI, testing and validation scripts remain Rust-first.

## Offline release-evidence consistency gate

`src/release_gate.rs` provides `accept_verified_passing_evidence` and a Rust negative-test matrix. Its `ObservedRun` must be filled **only by a separately authenticated, read-only GitHub/artifact verifier**, never from report JSON. It checks a repository allowlist, canonical run URL, exact source/fixture hashes, status, case counts, guest execution, verified artifact digests, independent review and protected-branch evidence. The test suite uses synthetic observations and **does not implement GitHub API retrieval, prove signatures, execute a backend, or certify any production release**. It is fail-closed while those observations are missing.

## Read-only GitHub metadata probe

`src/github_probe.rs` and `src/bin/github_metadata_probe.rs` now fetch GitHub metadata using authenticated `gh api -X GET` for a **fixed, validated owner/repository**. The probe checks PR head, Git commit and exact tree, completed run/attempt and job/steps, exact-head independent reviewer approval, branch protection, and the existence of matching artifact digest **metadata**. The repo and endpoints are built from trusted CLI inputs, never from a report's untrusted URL. It fails closed when GitHub returns 403/404, when listing pages are truncated, or on mismatch. Rust negative tests use a mock read-only API.

Example (run IDs, commit/tree SHAs, and job/step names must be independently known):
```sh
cargo +1.88.0 run --locked --bin github_metadata_probe -- \
  benefactor-cc/ci 5 42 <commit_sha> <tree_sha> \
  .github/workflows/conformance.yml guest-tests ORESoftware guest-receipt \
  'Run cargo +1.88.0 test --locked --all-targets --verbose' 'Run guest tests'
```

**Important:** this probe deliberately exits nonzero even when all GitHub metadata checks pass: status remains `NOT_CERTIFIED` until an independent download and hash verification of artifact bytes, authenticated guest execution receipts, and review/protection provenance are implemented. A GitHub artifact's `digest` field alone does not prove its bytes; successful CI is not proof of backend guest execution. No repository mutation is performed by the probe.
