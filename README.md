# Oreslang compiler parity testing

A shared cross-backend conformance corpus for Java/GraalVM, LLVM, WebAssembly, .NET Native AOT, and the JavaScript transpiler.

The suite is deliberately fail-closed. The same `.ores` bytes must be presented to every backend. Unsupported features, infrastructure failures, unclassified negative diagnostics, or backend-specific source rewrites are never counted as parity passes.

The reference-grounded 36-program feature ladder covers scalar execution, control flow, diagnostics, ownership, cleanup/async/channels, actor isolation/lifecycle, and Java-reference static-select delimiter parity. A second whole-program layer under `checks/<lowercase-name>/` adds 16 positive multi-file checks plus 2 negative syntax probes. Together they form 54 logical cases and 270 backend/case cells. Programs 32-36 are canonical AttoLang `.atto` sources; extensionless imports resolve to exactly one of `name.atto` or legacy `name.ores` and fail as ambiguous when both exist.

Every positive multi-file entry point imports exactly three local source files, performs in-program assertions, and emits a stable `PARITY_OK|<name>|` prefix. The Rust runner recursively fingerprints the complete reachable import graph, including cycles, so a result is tied to all source bytes involved.

See `docs/TEST_MATRIX.md`, `docs/EXECUTION.md`, `docs/BACKEND_ADAPTERS.md`, `manifest.json`, `checks/manifest.json`, and `checks/README.md`.

Tooling follows the Oreslang Rust-only tooling policy; Python is prohibited.

For pinned Java reference execution and compiler/runtime identity receipts, see
[`docs/JAVA_REFERENCE.md`](docs/JAVA_REFERENCE.md). Reference matches remain
exploratory until their syntax and semantic oracles are independently validated.
