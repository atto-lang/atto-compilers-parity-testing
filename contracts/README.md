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
