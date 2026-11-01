# Five-backend pure-i64 parity hardening — 2026-11-01

## Contract and scope

The immutable common-source contract at `contracts/parity/portable-scalar-v1-20261101.json`
uses Git **blob SHA `001e8894cbeb87509878a4de36263a1a327805b1`** on
each of the five compiler PR branches below. This contract has 16 positive
signed-i64 expression outcomes and six deliberate *compile-reject* cases:
integer division, `^`, `^^`, `%`, and positive/negative out-of-range i64
literals. Identical source bytes, not rewritten backend variants, are required.
`%`, `^`, `^^` are removed operators, not legacy compatibility routes.

| Backend | Candidate PR | Snapshot PR head (do not reuse after head moves) | Source contract blob | Executed parity |
| --- | --- | --- | --- | --- |
| Java/Graal | oreslang-source.java #2395 | `2669d9dc9b2af9d5f8d591ac8e3012317ea5d8f3` | `001e889...` | **Unverified** |
| LLVM | oreslang-source.llvm #1252 | `65191b281f1f4fc03309e7b9caa21ddad43218ae` | `001e889...` | **Unverified** |
| Wasm | oreslang-source.wasm #218 | `b536ad006623d83541d1365bc30ea697d693289e` | `001e889...` | **Unverified** |
| .NET | oreslang-source.net #279 | `9ebb29895660f19f7c87dba9ff1d6eb5ad10881b` | `001e889...` | **Unverified** |
| Cranelift | oreslang-source.cranelift #21 | `9f68f6b65a537da27fb2627db09238a5aca4dd94` | `001e889...` | **Unverified** |

The Cranelift draft now has a real-JIT CLI test source that **would execute**
all 16 positive and six negative vectors from its exact copied contract. It
also contains byte-identical program files 34–41 and a Git-blob gate for each.
These are authored tests only: do **not** promote their semantic status until
nonzero-step runner receipts prove compiler and guest execution.

## Additional shared guest candidates (manifest IDs 42–44)

- **42**: `int x = 7; const y = x * 6; return y;`
- **43**: `const int x = 14; const y = x * 3; return y;`
- **44**: `const x: int = -(-42); return x;`

Each is a standalone native `.ores` program returning **42**, with no
backend-specific rewrite. All are marked `candidate` until each compiler
processes identical bytes and produces a verified guest value, or a distinct
failure is logged as a gap. Each tests a separate immutable-by-default
declaration spelling. They are not yet Java/LLVM/Wasm/.NET executable parity
proof just because Cranelift accepts the spelling.

## Evidence separation and actionable gates

1. **Policy and provenance**: verify approved source-org runner, current
   exact PR head/tree, copied Git blob, compiler executable/toolchain identity,
   and no unauthorized private-source cross-organization transfer.
2. **Runner admission**: record actual runner ID, run attempt, job-to-step
   binding, nonempty executed steps, checkout SHA, and source-tree SHA.
   Jobs with `steps=[]` or `steps=null` provide **zero test evidence**.
3. **Compiler-backend evidence**: Java must run the JVM/Graal guest or JUnit
   compiler lane, LLVM must compile/link/execute the native guest, Wasm must
   use an exact SHA-256 pinned LLVM-to-wasm32 compiler with
   `ORESLANG_SCALAR_PARITY_REQUIRED=1` and a real isolated guest, .NET must
   verify its structured frontend and emitted CIL/NativeAOT execution, and
   Cranelift must run `cargo fmt --all -- --check`,
   `cargo clippy --locked --all-targets -- -D warnings`, and
   `cargo test --locked --all-targets` including `portable_scalar_v1` and
   `portable_scalar_intersection`.
4. **Oracle and negative cases**: check exact signed i64 bytes/values, not
   binary64-rounded results; inspect diagnostic category and ensure no
   guest artifact was published on compiler-negative source. Track no-op,
   skipped, host-only, and source-only checks separately.
5. **Independent review**: only after the same test contract is verified
   on all five *different* compiled guest paths may a row be labeled
   `verified_pass`. Unsupported or infrastructure-blocked cannot be PASS.

Fresh sampled source-org workflow failures for Java (`fast-ci`
run `41333009445`), LLVM (`llvm-smoke` run `41332794071`), Wasm
(`WebAssembly scalar backend checks` run `41332947806`), and .NET
(`Strict .NET Linux x64 Native AOT` run `41332796906`) have
`steps=null` in returned job metadata. They do not demonstrate executed
tests. Re-check the *latest head* and job steps when runner admission is
repaired. Cranelift's new contract test head had no associated returned
workflow runs when first inspected.

**Not established:** generic/ADT/typeclass support, actors, separate
heaps, structured concurrency, async IO, native stdlib test harness, or
general scalar checked-cast parity. Follow the separate Java #2405, LLVM
#1256, .NET #282, Wasm #219 and Cranelift #21 work without crediting
one backend's source-only cases to the others.
