# Target-aware differential runner migration (candidate)

2026-11-25. This is a migration plan for the existing parity runner, not a new normative language spec, a second target-profile registry, or an execution receipt.

## Observed gap at PR #38 head e17bf0033497f74f49f9ff5f8e8fca3343e2c348

The legacy manifest and Rust runner expose six backend identifiers. The adapter documentation calls the `wasm` lane an LLVM-to-Wasm bridge, while the current [architecture matrix](https://github.com/atto-lang/.github/blob/main/docs/compiler-feature-matrix.md) and [Wasm README](https://github.com/atto-lang/atto-source.wasm/blob/main/README.md) distinguish the Rust-authored source compiler from the Wasm runtime integration. PR #39 instead requires Forge and makes JS optional; PRs #36 and #44 also modify the legacy roster and runner. These differences must be reconciled rather than merged independently. Preserve legacy manifest/CLI compatibility until the replacement executes real guests.

## Candidate lane identities (not capability assertions)

| Lane | Compiler owner | Artifact/entrypoint | Runtime owner or engine |
| --- | --- | --- | --- |
| java-jvm | atto-source.java | JVM guest | JVM |
| java-native | atto-source.java | Graal Native Image executable | native |
| net-cil | atto-source.net | CIL | CoreCLR |
| net-native | atto-source.net | NativeAOT executable | native |
| llvm-native | atto-source.llvm | native executable | native |
| rs-wasm | atto-source.rs | generated Wasm | pinned Wasm engine |
| rs-wasm-worker | atto-source.rs | same generated Wasm | atto-source.wasm worker integration |
| llvm-wasm-bridge | atto-source.llvm | generated Wasm, where supported | atto-source.wasm bridge |
| js-esm | atto-source.js | generated ESM | pinned JS engine |
| forge-native | atto-source.forge | native artifact | native |
| cranelift-jit | atto-source.cranelift | compiled function | pinned JIT host |
| cranelift-aot | atto-source.cranelift | native artifact, when implemented | native |
| atto-bootstrap | atto-source.atto | pinned stage-0 seed output | external compiler; not self-hosted |
| hdl-rtl | atto-source.hdl | RTL | simulator / formal equivalence |
| realtime-profile | atto-source.realtime | admitted target artifact | declared deterministic target |

Separate compiler identity (repository, commit, root tree, binary SHA-256, implementation lineage), execution lane (target, artifact type, mode, engine/runtime commit, version, binary identity), and feature applicability. Multiple hosts of one compiler are multiple runtime receipts, never extra independent compiler votes. A wasm32 build of attoc is a compiler-host variation, not another guest target. JS source-to-source compilation is a legitimate lane and never a native AOT claim. HDL and realtime use explicit constrained profiles. The Atto seed is not a self-hosting result. The authoritative profile declarations remain in atto-target-profiles, and all portable semantics in atto-spec.

## Required next runner work

1. Introduce a versioned additive lane manifest and adapter selection; do not silently change the legacy `wasm` alias from the LLVM bridge to Rust emission. Require explicit lane identity for new receipts. Fail closed on unknown, ambiguous, duplicate or inconsistent compiler/runtime/target identities.
2. Bind exact source graph (including imports), spec and profile pins, compiler head/tree and executable, generated artifact digest, execution toolchain, mode and original guest observations. Verify artifact after compilation and around execution; a caller-supplied digest or successful compile alone is insufficient. Retain #38's mandatory Wasm digest and #49's strict guest stdout rules.
3. Use a type-aware result oracle (full signed integer values, booleans, stdout, failure category and side effects). Process exit codes alone truncate wide integers; JS BigInt, JVM, CIL, Wasm i64 and native result transport must be lossless. Reject forged diagnostics and reject expected compile-failure cases that emit executable output. Compare the same unchanged .atto source and inputs against the spec oracle, not majority agreement. Retain #39's wide-integer and Forge requirements, #36/#44's unique security and source tests after checking current syntax against atto-spec (legacy removed-operator claims are not automatically current).
4. Distinguish PASS, FAIL, UNSUPPORTED, NOT_EXECUTED from profile non-applicability. Unsupported required core behavior is a gap; a deliberate out-of-profile case passes only its independently specified rejection oracle. Do not turn zero-step CI, host-only/fake-adapter tests, stage-0 models or missing adapters into guest success.
5. Begin with signed integer limits/overflow/division, exact stdout, immutable binding, nominal ADTs and import graph visibility/cycles. Then expand borrow/move/lifetime, tasks and actor isolation/scheduling, serialization and stdlib. Reuse authoritative source blobs from atto-platform-conformance; no per-target source rewriting.
6. Reconcile #36, #38, #39, #44 and #49 in a current-main integration branch only after identifying unique tests, reviewing full diffs and checking CI/holds. Keep #55 and #19's feature-specific ledgers complementary. No merge or promotion is authorized by this plan.

## Required adversarial harness cases

- A Rust compiler receipt cannot satisfy the independent LLVM or Java compiler cell; a Wasm host receipt cannot claim a second independent frontend.
- A JS ESM artifact cannot claim native AOT, a CLR DLL cannot claim NativeAOT, and JVM guest mode is not Graal Native Image.
- Reject a compiler binary mismatch, source graph mismatch, wrong runtime mode, artifact digest mismatch, duplicate lane and fake successful zero-step job.
- Distinguish 42 from 298, -1 from 255, and 9007199254740993 from 9007199254740992; do not compare via JS Number or modulo-256 process exit code.
- Missing required compiler or unsupported core case blocks the gate. HDL/runtime-specific rejection and independent security proof are separate from the portable source gate.

Status: documentation-only candidate; no compiler or guest execution, no change to the legacy runner, no source export, no promotion or merge. Next step is a reviewed implementation and exact-head tests.
