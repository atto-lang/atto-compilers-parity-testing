# Oreslang compiler parity testing

A shared cross-backend conformance corpus for the six required `oreslang-source.*` backends: Java/GraalVM, LLVM, WebAssembly, .NET Native AOT, Forge and Cranelift. The JavaScript transpiler is a separately selectable, optional seventh target.

The suite is deliberately fail-closed. The same `.ores` bytes must be presented to every backend. Unsupported features, infrastructure failures, unclassified negative diagnostics, or backend-specific source rewrites are never counted as parity passes.

The 31-program original feature ladder covers scalar execution, control flow, diagnostics, ownership, cleanup/async/channels, actors, and Java-reference static-select delimiters. Four unvalidated signed-i64 boundary/metamorphic programs (32–35) extend this ladder. One actor-lifecycle `.atto` reference case (36) and five canonical AttoLang `.atto` router-derived programs (37–41), each reduced from an `oreslang-http-routing` construct, record the native gaps that keep the router off LLVM. Extensionless imports resolve to exactly one of `name.atto` or legacy `name.ores` and fail as ambiguous when both exist. A second whole-program layer under `checks/<lowercase-name>/` adds 16 positive multi-file checks plus 2 negative syntax probes. Together they form **59 logical cases and 354 required backend/case cells**, with a separately selectable 59-case JavaScript inventory.

Every positive multi-file entry point imports exactly three local source files, performs in-program assertions, and emits a stable `PARITY_OK|<name>|` prefix. The Rust runner recursively fingerprints the complete reachable import graph, including cycles, so a result is tied to all source bytes involved.

See `docs/TEST_MATRIX.md`, `docs/EXECUTION.md`, `docs/BACKEND_ADAPTERS.md`, `docs/CI_EXECUTION_EVIDENCE.md`, `manifest.json`, `checks/manifest.json`, and `checks/README.md`.

AttoLang specification alignment is tracked separately in [docs/SPEC_ALIGNMENT.md](docs/SPEC_ALIGNMENT.md) and [the source-only registry](contracts/spec-alignment-v1.json). New language sources should use `.atto`; the existing `.ores` corpus is retained unchanged as a versioned legacy compatibility baseline. Registry links are not execution receipts and do not alter the 53 × 6 inventory.

CI execution evidence is tiered separately from corpus status. Source-side exact-run
verification begins at `RUNNER_EXECUTED`; compiler/guest tiers require stronger
artifact/provenance evidence, and `ORES_GUEST_EXECUTED` must also bind to the
matching executed parity-report cell.

Tooling follows the Oreslang Rust-only tooling policy; Python is prohibited.

Default inventory audits the six required backends. Use `--backend forge` to select Forge or `--backend js` for optional transpiler inventory/exploration. Forge's `run` command currently prints a scalar result via its interpreter and does **not** satisfy this runner's guest exit-code oracle without a reviewed adapter. Forge's constant-return assembly tests are also not full-program native execution proof. The new arithmetic candidates are deliberately **not validated**. No new backend execution is claimed.

For pinned Java reference execution and compiler/runtime identity receipts, see
[`docs/JAVA_REFERENCE.md`](docs/JAVA_REFERENCE.md). Reference matches remain
exploratory until their syntax and semantic oracles are independently validated.

Cranelift is also a required independent target, alongside LLVM, Java, Wasm, .NET and Forge; JavaScript remains optional. The 59-case default audit now yields 354 non-executed cells. Cranelift uses ORES_PARITY_CRANELIFT_COMPILE and ORES_PARITY_CRANELIFT_RUN. Its scalar JIT has no reviewed separate artifact adapter for this corpus; unavailable or unsupported source remains a gap, never a pass.
