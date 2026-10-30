# CI execution evidence contract

Cross-backend parity has two independent questions:

1. did the exact GitHub Actions run attempt actually execute on a runner?
2. what level of backend/compiler/guest work did that execution prove?

The source repositories emit `oreslang-parity-execution-evidence-v1` only after
their read-only verifier has bound the exact private repository identity, source
commit/tree, trusted workflow file/blob, run ID, **run attempt**, job IDs,
positive runner IDs/nonblank runner names, and actually completed
non-housekeeping steps.

The parity corpus validates those bundles with:

```bash
cargo run --manifest-path tools/parity-runner/Cargo.toml \
  --bin evidence-contract -- evidence/fixtures/synthetic-runner-executed.json
```

The fixture above is intentionally synthetic. Passing it proves only that the
validator accepts/rejects the schema correctly; it is never execution evidence.

## Claim tiers

The tiers are monotonic and fail closed.

| Tier | Required evidence |
| --- | --- |
| `RUNNER_EXECUTED` | Exact repo/head/tree/workflow/run/attempt, positive runner identity on the same job, and at least one completed meaningful non-housekeeping step. |
| `HOST_TESTS_EXECUTED` | Everything above plus a meaningful step categorized `host_compiler_tests`. |
| `NATIVE_COMPILER_BUILT` | Everything above plus `native_compiler_build`, compiler identity/SHA-256, and an unexpired artifact whose SHA-256 matches the compiler provenance. |
| `ORES_GUEST_EXECUTED` | Everything above plus `compiled_ores_guest_runtime_execution`, exact guest source/artifact SHA-256, and exact exit/stdout oracle equality. |

A workflow/job conclusion alone never establishes a tier. A listed but
unstarted step, zero-step job, missing runner identity, `runner_id=0`, blank
runner name, housekeeping-only execution, stale/expired referenced artifact, or
oracle mismatch is rejected.

The source-side verifier currently emits only `RUNNER_EXECUTED`. Its meaningful
steps are categorized `indeterminate` deliberately. A later backend-specific
verifier must provide the stronger category/provenance fields before a parity
report may claim host tests, native compiler build, or compiled guest execution.

## Backend identity

The five required source identities are pinned numerically:

| Source evidence key | Repository | Repository ID | Parity target |
| --- | --- | ---: | --- |
| `java` | `ores-truffle-oreslang/oreslang-source.java` | 1400792274 | `java` |
| `llvm` | `ores-truffle-oreslang/oreslang-source.llvm` | 1410897874 | `llvm` |
| `net` | `ores-truffle-oreslang/oreslang-source.net` | 1419704375 | `dotnet` |
| `wasm` | `ores-truffle-oreslang/oreslang-source.wasm` | 1419858954 | `wasm` |
| `forge` | `ores-truffle-oreslang/oreslang-source.forge` | 1419703674 | `forge` |

The explicit `net` → `dotnet` normalization prevents accidental omission or
double-counting caused by naming drift.

## Evidence is not parity by itself

`RUNNER_EXECUTED` proves runner execution and Git provenance only.
`HOST_TESTS_EXECUTED` proves only the categorized host test step.
`NATIVE_COMPILER_BUILT` proves only the compiler-build provenance described in
the bundle.

Only `ORES_GUEST_EXECUTED` is eligible to support a compiled Oreslang guest
runtime parity claim, and even then the parity runner must independently match
the same logical corpus case/backend cell and its source hash/oracle.

Inventory-only `not_run` records, interpreter-only Forge results, source-free
canaries, carrier harness tests, and structural manifest checks must never be
reported as compiled guest parity.
