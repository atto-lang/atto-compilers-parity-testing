# Oreslang compiler parity testing

A shared cross-backend conformance corpus for the six required `oreslang-source.*` backends: Java/GraalVM, LLVM, WebAssembly, .NET Native AOT, Forge and Cranelift. The JavaScript transpiler is a separately selectable, optional seventh target.

The suite is deliberately fail-closed. The same `.ores` bytes must be presented to every backend. Unsupported features, infrastructure failures, unclassified negative diagnostics, or backend-specific source rewrites are never counted as parity passes.

The 31-program original feature ladder covers scalar execution, control flow, diagnostics, ownership, cleanup/async/channels, actors, and Java-reference static-select delimiters. Four unvalidated signed-i64 boundary/metamorphic programs (32–35) extend this ladder. A second whole-program layer under `checks/<lowercase-name>/` adds 16 positive multi-file checks plus 2 negative syntax probes. Together they form **53 logical cases and 318 required backend/case cells**, with a separately selectable 53-case JavaScript inventory.

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

Cranelift is also a required independent target, alongside LLVM, Java, Wasm, .NET and Forge; JavaScript remains optional. The 53-case default audit now yields 318 non-executed cells. Cranelift uses ORES_PARITY_CRANELIFT_COMPILE and ORES_PARITY_CRANELIFT_RUN. Its scalar JIT has no reviewed separate artifact adapter for this corpus; unavailable or unsupported source remains a gap, never a pass.

## Repository ownership and migration (2026-11-08)

**Public name:** AttoLang (formerly Oreslang). This remains `ores-truffle-oreslang/oreslang-compilers-parity-testing` until a real authorized repo transfer. Preserve established `.ores` inputs and old tool/package names; do not claim feature parity from a branding or README change.

- **Scope:** compiler-pinned cross-backend source corpus, differential tests and negative compiler admission checks.
- **Not this repo:** implementing language features or fabricating guest results from host-only CI.
- **Where code belongs:** [atto-lang](https://github.com/atto-lang) is the canonical home for specs, compilers, portable core/std, developer tooling, conformance and ecosystem examples. [atto-co](https://github.com/atto-co) separately governs deployment, identity, billing, tenant hosting and other managed infrastructure. Existing `atto-*` repos are not automatically transferred source identities.
- **Planned first-stage destination (not approved/executed):** `atto-lang/oreslang-compilers-parity-testing`, under the same name and GitHub numeric repository ID. Direct legacy-org → `atto-co` transfers are not authorized in this wave; future product carve-outs require separate review.

**Migration checklist:** announce precise repo/head/rollback owner in [coordination issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26); use private [migration plan #25](https://github.com/ores-truffle-oreslang/.github/pull/25). Obtain source/target admins and independent security signoff, verify target availability/protections/access, capture numeric repo ID and exact head SHA/tree, PRs/releases, Actions/runners/secrets, packages, deploy keys, GitHub Apps, webhooks, Pages and consumers. Never clone private source into a new org to bypass approval. Confirm stable identity/history/tree after authorized GitHub transfer and rerun exact-head CI at destination. **Zero-step jobs (`steps=[]` or `null`) are `NOT_EXECUTED`; host-only or source-free probes are not compiler-backed AttoLang guest receipts.** Move package/import references and names only after validated cutover.
