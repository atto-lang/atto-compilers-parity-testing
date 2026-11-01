# Oreslang compiler parity testing

A shared cross-backend conformance corpus for Java/GraalVM, LLVM, WebAssembly, .NET Native AOT, and the JavaScript transpiler.

The suite is deliberately fail-closed. The same `.ores` bytes must be presented to every backend. Unsupported features, infrastructure failures, unclassified negative diagnostics, or backend-specific source rewrites are never counted as parity passes.

The reference-grounded 31-program feature ladder covers scalar execution, control flow, diagnostics, ownership, cleanup/async/channels, actor isolation/lifecycle, and Java-reference static-select delimiter parity. A second whole-program layer under `checks/<lowercase-name>/` adds 16 positive multi-file checks plus 2 negative syntax probes. Together they form 59 logical cases and 354 backend/case cells (including the experimental Cranelift target). These are inventory cells, not executed proof of backend parity.

Every positive multi-file entry point imports exactly three local source files, performs in-program assertions, and emits a stable `PARITY_OK|<name>|` prefix. The Rust runner recursively fingerprints the complete reachable import graph, including cycles, so a result is tied to all source bytes involved.

See `docs/TEST_MATRIX.md`, `docs/EXECUTION.md`, `docs/BACKEND_ADAPTERS.md`, `manifest.json`, `checks/manifest.json`, and `checks/README.md`.

Tooling follows the Oreslang Rust-only tooling policy; Python is prohibited.

For pinned Java reference execution and compiler/runtime identity receipts, see
[`docs/JAVA_REFERENCE.md`](docs/JAVA_REFERENCE.md). Reference matches remain
exploratory until their syntax and semantic oracles are independently validated.

## 2026-11-01 six-backend inventory and Cranelift scalar intersection

The corpus now registers 41 legacy source programs, 16 multi-file checks and
two negative probes: **59** logical cases over six target backends
(Java, LLVM, Wasm, .NET, JS, and experimental Cranelift). These **354 cells
are only inventory**, and are `not_run` unless explicitly executed.

Programs 34 through 41 form the additional restricted checked-i64 scalar
intersection. Cranelift currently supports arithmetic expressions, not
control-flow, imports, actors or the remaining Oreslang semantics. Do **not**
claim language-wide Cranelift parity from these eight cases.

Use a separately built, trusted `orescranelift` executable on PATH:
```sh
export ORES_PARITY_CRANELIFT_COMPILE='orescranelift ir {source}'
export ORES_PARITY_CRANELIFT_RUN='orescranelift run {source}'
cargo run --locked --manifest-path tools/parity-runner/Cargo.toml -- \
  --execute --allow-unvalidated --backend cranelift \
  --case 34 --case 35 --case 36 --case 37 \
  --case 38 --case 39 --case 40 --case 41 \
  --output cranelift-scalar-candidates.json
```

For scalar cases the Cranelift CLI must exit zero, emit exactly the decimal
signed-i64 result plus one LF on stdout, and emit no stderr. Other backends
must return the expected exit code; Cranelift stdout can never substitute for
a Java/LLVM exit value. The runner deliberately exits **nonzero** if candidate
cases match, because exploratory matches are not parity certification.
An unconfigured adapter also fails closed. Actual evidence additionally needs
compiler and guest artifact identity, exact Git tree, executed nonzero-step
CI, review, and native/NativeAOT receipts where applicable. No unapproved
private-source cross-organization mirroring is authorized.
