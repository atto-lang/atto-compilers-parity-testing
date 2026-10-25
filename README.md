# Oreslang compiler parity testing

A shared cross-backend conformance corpus for Java/GraalVM, LLVM, WebAssembly, .NET Native AOT, and the JavaScript transpiler.

The suite is deliberately fail-closed. The same `.ores` bytes must be presented to every backend. Unsupported features, infrastructure failures, unclassified negative diagnostics, or backend-specific source rewrites are never counted as parity passes.

The reference-grounded 30-program feature ladder covers scalar execution, control flow, diagnostics, ownership, cleanup/async/channels, and actor isolation/lifecycle. A second whole-program layer under `checks/<lowercase-name>/` adds 16 positive multi-file checks plus 2 negative syntax probes. Together they form 48 logical cases and 240 backend/case cells.

Every positive multi-file entry point imports exactly three local source files, performs in-program assertions, and emits a stable `PARITY_OK|<name>|` prefix. The Rust runner recursively fingerprints the complete reachable import graph, including cycles, so a result is tied to all source bytes involved.

See `docs/TEST_MATRIX.md`, `docs/EXECUTION.md`, `manifest.json`, `checks/manifest.json`, and `checks/README.md`.

Tooling follows the Oreslang Rust-only tooling policy; Python is prohibited.
