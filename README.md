# Oreslang compiler parity testing

A shared cross-backend conformance corpus for Java/GraalVM, LLVM, WebAssembly, .NET Native AOT, Cranelift JIT, and the JavaScript transpiler.

The suite is deliberately fail-closed. The same `.ores` bytes must be presented to every backend. Unsupported features, infrastructure failures, unclassified negative diagnostics, or backend-specific source rewrites are never counted as parity passes.

The reference-grounded 51-program feature ladder covers scalar execution, control flow, diagnostics, ownership, cleanup/async/channels, actor isolation/lifecycle, and Java-reference static-select delimiter parity. A second whole-program layer under `checks/<lowercase-name>/` adds 16 positive multi-file checks plus 2 negative syntax probes. Together they form 69 logical cases and 414 backend/case cells.

Every positive multi-file entry point imports exactly three local source files, performs in-program assertions, and emits a stable `PARITY_OK|<name>|` prefix. The Rust runner recursively fingerprints the complete reachable import graph, including cycles, so a result is tied to all source bytes involved.

See `docs/TEST_MATRIX.md`, `docs/EXECUTION.md`, `docs/BACKEND_ADAPTERS.md`, `manifest.json`, `checks/manifest.json`, and `checks/README.md`.

Tooling follows the Oreslang Rust-only tooling policy; Python is prohibited.

For pinned Java reference execution and compiler/runtime identity receipts, see
[`docs/JAVA_REFERENCE.md`](docs/JAVA_REFERENCE.md). Reference matches remain
exploratory until their syntax and semantic oracles are independently validated.

## GLM/MoE shared-source frontier (cases 32–51)

Twenty new native `.ores` cases exercise fixed-size MoE logic with **identical
source files** for Java/Graal, LLVM, WASM, .NET and Cranelift. Cases 32–48 cover integer
top-1 ranking/ties, no-fallback capacity, assignment-count predicates,
top-k bounds, version fencing, cancellation, and exact step order.
Cases 49–51 are a **floating-point frontier**: fixed-gate prediction,
analytic weight gradient and an exact-binary-fraction SGD loss reduction.

All 20 cases are `candidate`, not compiler-verified. A failed compile,
unsupported lowering, skipped run, zero-step Actions job, or missing approved
toolchain remains a **parity gap**. In particular the current Wasm bridge
delegates compilation to a pinned LLVM compiler and documents unavailable
source float parity, while .NET NativeAOT's source subset must be checked
through actual CIL + native host execution. No backend-specific rewrites
of the guest source or declaration of success from host-only mathematical
oracles are allowed.

The Rust `glm_moe_corpus.rs` checks the source inventory and independent
mathematical expectations; those tests **do not execute any Oreslang guest**.
See [GLM backend acceptance](docs/GLM_MOE_BACKEND_PARITY.md).

Cranelift is a distinct, required sixth corpus backend, not an LLVM alias or a passing host oracle. A missing Cranelift compiler is an explicit parity gap. See [Cranelift admission gates](docs/FIVE_BACKEND_ADMISSION_GATE.md).
