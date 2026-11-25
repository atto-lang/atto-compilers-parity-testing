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

## Signed guest ZIP artifact verification (Rust)

The new src/artifact_receipt.rs module downloads artifact ZIPs through authenticated, host-pinned GitHub CLI GET calls, hashes the **actual ZIP bytes** using Rust SHA-256, and validates a bounded two-file ZIP containing guest-receipt.json and guest-stdout.bin. It rejects unknown ZIP paths, excessive decompression, mismatched output hashes, stale source/fixture/run/attempt/job, non-guest results, failed cases, missing challenge nonces, and replayed receipts in one audit batch. For signed provenance, GhProvenance requires a cryptographically verified SLSA attestation with repository, source digest and signer-workflow constraints. The production verifier fails closed if GitHub provenance cannot be verified; test-only fake signers cannot prove deployment readiness.

**Still NOT_CERTIFIED:** a signed ZIP containing a receipt does not independently prove that a trusted guest runner executed anything. Backend execution and cross-batch replay protection require a trusted signer/runner and persisted state. Existing source-main protection and independent-review checks remain separate hard gates. No repository mutation is performed by the verifier.

## Canonical verifier consolidation and persistent replay gate

The canonical `github_probe.rs` and `artifact_receipt.rs` are the only
implementation paths for the new security checks. Strict duplicate-safe
Rust decoding applies to untrusted GitHub API metadata, SLSA verification
responses and guest receipts. The probe rejects superseded approvals, missing
runner identities, wrong artifact head SHAs and empty artifacts.

A separate local replay journal is now **mandatory** for the combined
`guest_artifact_probe` CLI: set `ATTO_REPLAY_JOURNAL_DIR` to an existing,
owner-private directory (mode 0700 on Unix) **outside source repositories**.
An atomic `create_new` receipt marker with mode 0600 prevents acceptance
of the same source/run/attempt/job/backend/feature/nonce across separate
process invocations. Missing, unsafe or unwritable journals fail closed.
This mutates *only* the explicitly configured local journal; it never
changes GitHub repositories, policies, or source trees.

The optional `fixture-set.json` third ZIP member permits independent
SHA-256 validation of the actual fixture bytes and enforces exact executed
case count. Older two-file receipts are still parseable, but report
`fixture_bytes_verified=false` and must never be treated as certified
fixture execution. Verified bytes, a valid SLSA attestation and replay
prevention still **do not prove the guest runtime actually executed**.
Release status remains `NOT_CERTIFIED` until independently trusted
runner/guest execution evidence is established.
