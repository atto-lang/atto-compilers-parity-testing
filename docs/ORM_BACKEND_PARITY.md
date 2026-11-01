# ORM schema parity requirements: Java, LLVM, WASM and .NET

**NOT VERIFIED**: no exact-head backend compiler/guest receipts have been produced for
`checks/orm-schema-backend-parity/main.ores`. This is a standalone, unchanged
four-file native Oreslang source graph, not a substitute for the full ORM.

The guest checks nominal class construction, immutable field reads, stable-ID
preservation after physical rename, type changes, required-column backfill,
destructive drops, no-op diffs and denial of any production schema DDL,
even if parity/complete-catalog/operator flags all appear set.

| Backend | Implementation boundary | Gate before saying parity |
| --- | --- | --- |
| Java/GraalVM | reference frontend and runtime | pinned source/guest exact-run; stdout receipt |
| LLVM | AOT via compile-aot.sh | pinned compiler build, native binary, guest output |
| WASM | LLVM-to-Wasm bridge | full module graph support + SHA-bound .wasm admission + execution |
| .NET | source compiler/NativeAOT host | full module graph + real NativeAOT publish and execution |

Every backend must ingest **identical source bytes and graph SHA256**.
No AST rewriting for a backend. Failed compilation, unsupported import graphs,
zero-step runners, host-only checks, or declared unsupported capability are
PARITY GAPS—not passes. An actual positive result must reach
`PARITY_OK|orm-schema-backend-parity|42` after in-guest checks with exit 0.

The full ORM native feature list further includes actor-owned async PostgreSQL
connections, checked binds and OIDs, cross-tenant RLS, cancellation/rollback,
reviewed TypeSpec↔JSON Schema and DPM catalog parity. Those features are
**NOT** exercised by this tiny language-contract probe. No database is
opened and no migration is executed.

Source-of-truth PR: `ores-truffle-oreslang/postgres-orm.ores` PR #13;
source must be compared at a pinned reviewed SHA before using it as
compiler correctness evidence. In particular the portable feature probe
does not certify the entire ORM implementation or production readiness.

Observed adapter detail (2026-11-01): the WASM native bridge accepts only
core i64 entrypoints and prints a JSON result; the ORM check is a four-file
void-entry guest that prints a PARITY_OK line. Artifact digest pinning now
exists in the Rust adapter but does NOT provide an equivalent stdout/guest
ABI. .NET has a `compile-modules` command limited to verified int/bool
module graphs; it does NOT prove nominal class/constructor lowering, nor
NativeAOT execution for the ORM fixture. Preserve both as open failures.
