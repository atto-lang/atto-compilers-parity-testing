# Atto std/env — differential receipt admission v1

This tool is the **independent receipt sanity-checking companion** to
[atto-platform-conformance PR #576](https://github.com/atto-lang/atto-platform-conformance/pull/576).
The authoritative behavioral contract is `contracts/std-env-v1/contract.json`
in that repo, pinned here to Git blob SHA
`e20331b2c17e29eefdd2ba04ea5f750c0b56eff1`.
It contains 13 portable `.atto` cases and exact SHA identities for ten
fixtures. The tool never substitutes canonical shared `.atto` library source
for the 11 independently maintained backend implementations.

`status-ledger.json` initially records **NOT_EXECUTED** for all eleven
`atto-source.*` targets: Java, LLVM, .NET, Rust, Forge, Cranelift,
JavaScript, Wasm, Atto self-host, realtime and HDL. HDL synthesis is not a
runtime environment and must explicitly reject unsupported effects; self-host
and realtime are bootstrapping. No target may be omitted to improve coverage
percentages.

## Verification

```sh
node tools/std-env-v1/verify-receipts.mjs
node --test tools/std-env-v1/verify-receipts.test.mjs
```

This validates ledger structure and **eight adversarial examples**: fake PASS,
missing or duplicated backends, zero executed steps, host-only claims, malformed
GitHub run URLs, wrong contract SHA, changed fixture blobs, mismatched guest
output and leaking secret-bearing raw output.

A future claim of `passed` requires an independently pinned contract file as
second CLI argument, the compiler/source head SHA and root tree SHA, full
toolchain and target, GitHub Actions run URL with nonempty executed job steps,
all 13 case records with exact fixture Git blobs, actual guest compilation/
execution flags, controlled injection policies and exact stdout/error category.
The oversized-value case must fail at admission **before** any guest runs.

Even a well-formed candidate **never earns an authenticated PASS from this
validator alone**. The result is always
`CANDIDATE_REQUIRES_INDEPENDENT_GITHUB_PROVENANCE` until the trusted GitHub
integration retrieves and verifies the real run, jobs, steps, logs and
exact-tree provenance from GitHub, plus a separate signed/security-reviewed
source-to-carrier handoff if one was used. A private-source export is currently
blocked by the Java V3 protected-main and carrier approval gates recorded in
[atto-source.java#1788](https://github.com/atto-lang/atto-source.java/issues/1788).

`UNSUPPORTED` must be backed by an actual target restriction, not a missing
runner or incomplete compiler. Source-only tests, synthetic host tests, queued
or zero-step Actions and one-sided host adapters remain `NOT_EXECUTED` for
guest parity.

## Overlap handling

This new checker is separate from the legacy five-backend evidence schema and
the six-backend Rust parity-runner work. Do not widen or overwrite those
schemas implicitly. Related open PRs #19 and #49 cover broader receipt/oracle
security; this specialization is a forward-compatible eleven-target
`std/env` schema, and merging requires resolving their overlapping safeguards
and recording the disposition in all affected PRs.
