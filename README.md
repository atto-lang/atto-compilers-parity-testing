# Oreslang compiler parity testing

A shared cross-backend conformance corpus for six **required, independent** compilers: Java/GraalVM, LLVM, WebAssembly, .NET NativeAOT, Forge, and Cranelift JIT. The JavaScript transpiler is a separately selectable optional seventh adapter.

The suite is deliberately fail-closed. The same `.ores` bytes must be presented to every backend. Unsupported features, infrastructure failures, unclassified negative diagnostics, or backend-specific source rewrites are never counted as parity passes.

The reconciled 55-program feature ladder covers scalar execution, control flow, diagnostics, ownership, cleanup/async/channels, actor isolation/lifecycle, and Java-reference static-select delimiter parity. A second whole-program layer under `checks/<lowercase-name>/` adds 16 positive multi-file checks plus 2 negative syntax probes. Together they form **73 logical cases and 438 required compiler/case inventory cells**, plus 73 optional JavaScript inventory cells. These are not compiled-guest passes.

Every positive multi-file entry point imports exactly three local source files, performs in-program assertions, and emits a stable `PARITY_OK|<name>|` prefix. The Rust runner recursively fingerprints the complete reachable import graph, including cycles, so a result is tied to all source bytes involved.

See `docs/TEST_MATRIX.md`, `docs/EXECUTION.md`, `docs/BACKEND_ADAPTERS.md`, `manifest.json`, `checks/manifest.json`, `checks/README.md`, and `docs/CI_EXECUTION_EVIDENCE.md`.

Tooling follows the Oreslang Rust-only tooling policy; Python is prohibited.

For pinned Java reference execution and compiler/runtime identity receipts, see
[`docs/JAVA_REFERENCE.md`](docs/JAVA_REFERENCE.md). Reference matches remain
exploratory until their syntax and semantic oracles are independently validated.

## GLM/MoE shared-source frontier (20 cases: IDs 36–55)

Twenty native `.ores` cases exercise fixed-size MoE logic with **identical
source files** for Java/Graal, LLVM, WASM, .NET, Forge and Cranelift. The four GLM cases formerly numbered 32–35 have moved to 52–55 to preserve the signed-i64 boundary cases added on `main`. Cases 36–48 and 52–55 cover integer
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

Cranelift and Forge are distinct required compiler targets, not aliases for LLVM or passing host oracles. JavaScript remains optional. A missing Cranelift compiler is an explicit parity gap. See [Cranelift admission gates](docs/FIVE_BACKEND_ADMISSION_GATE.md).

## Executed evidence contract

Preserve `docs/CI_EXECUTION_EVIDENCE.md` and its pinned, independent `evidence-contract` binary and regression tests from `main`. A run with `steps=null` cannot establish even `RUNNER_EXECUTED`; a stepful host test cannot establish compiler-backed guest parity. Source-free alternate-org canaries do not authorize the transfer of private compiler/corpus source. A newly approved transfer must bind the current exact Git tree and destination and satisfy independent security review.
