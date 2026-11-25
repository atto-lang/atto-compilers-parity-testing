> **AI agent instructions (organization-wide):** Before working on code, issues, or pull requests, read and follow the [organization `AGENTS.md`](https://github.com/atto-lang/.github/blob/main/AGENTS.md), including its **merge-first stealth-development policy**. Honor any repository-local `AGENTS.md`, explicit holds, and GitHub-enforced rules.

# atto-compilers-parity-testing

Differential testing and behavioral parity verification across AttoLang compiler backends.

## Naming migration and language authority

This repository is maintained at [atto-lang/atto-compilers-parity-testing](https://github.com/atto-lang/atto-compilers-parity-testing); use `https://github.com/atto-lang/atto-compilers-parity-testing.git` for Git remotes and canonical `.atto` source examples. The normative language contract is [atto-spec](https://github.com/atto-lang/atto-spec), with executable guest evidence tracked by [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance). Follow current ownership, borrowing, effects, actor, and syntax rules there; do not infer implemented features from a README or unexecuted CI.

## Purpose

Compare same-program semantics across independent compiler/runtime backends.

## Repository responsibilities

- Keep interfaces, schemas, and behavior documented in a backend-neutral form where applicable.
- Use test fixtures and reproducible evidence to distinguish proposed behavior from implemented behavior.
- Record breaking changes, migration notes, and dependencies on other AttoLang components.

## Validation

Validation should rely on checked-in tests and exact-commit CI results. When reporting parity, distinguish **passed**, **failed**, **unsupported**, and **not executed**. Avoid inferring guest-runtime support from host-only tests.

## Status and development

This README records the intended role of this repository; it is not a claim of feature completeness. Consult the source tree and workflow files for actual implementation and verified build commands. Contributions should preserve compatibility with the canonical language contract.

## Ecosystem

- [AttoLang](https://github.com/atto-lang)
- [Language specification](https://github.com/atto-lang/atto-spec)
- [Cross-platform conformance](https://github.com/atto-lang/atto-platform-conformance)

## Repository ownership and contribution routing

**Belongs here:** Differential execution harnesses, normalizers, mismatch triage and regressions across independent AttoLang compilers.

**Belongs elsewhere:** Canonical semantics belong in atto-spec; primary feature fixtures and formal backend receipts belong in atto-platform-conformance.

**Security and portability boundary:** Compare identical source and equivalent flags/inputs; include diagnostic and rejection parity, deterministic replay and exact-head executed negative cases.

AttoLang is the canonical project name; new source, examples and documentation use `.atto` and the `atto-lang` repository organization. This README defines scope, not feature readiness.

**Acceptance:** name the exact source SHA/tree, runtime/toolchain, positive and negative checks and execution receipt before claiming support. Host-only tests and empty CI steps do not prove guest execution.

## Legacy provenance and transfer verification

The historical predecessor, `ores-truffle-oreslang/oreslang-compilers-parity-testing`, is archived; the current `atto-lang/atto-compilers-parity-testing` has a separate numeric GitHub repository ID. Preserve working legacy `.ores` fixtures and package/CLI aliases until their canonical `.atto` replacements are verified. The scope is compiler-pinned cross-backend source corpora, differential tests and negative compiler admission checks, not implementing language features or treating host-only CI as guest execution.

[atto-lang](https://github.com/atto-lang) owns language and portable tooling; [atto-co](https://github.com/atto-co) separately governs managed infrastructure, tenant hosting, identity and billing. Any future original-ID transfer or commercial carve-out requires separate owner authorization and independent security review. Inventory exact SHA/tree, numeric ID, access, protections, PRs, releases, packages, runners, secrets, deploy keys, apps, webhooks, Pages, consumers and rollback ownership; verify identity and retest after an authorized transfer. Never mirror private source without authorization. Zero-step jobs are NOT_EXECUTED; host/reference checks are not compiled guest evidence.

## Checked stdio and nominal ADT pattern parity (draft)

The draft [stdio/on-is parity contract](docs/stdio-adt-is-parity.md) defines the 11-backend target matrix (`contracts/v1/stdio-adt-parity-v1.json`), Rust anti-forgery tests (`tests/stdio_adt_parity.rs`), and a fail-closed **actual compiler + compiled guest** runner (`tools/run-stdio-conformance.mjs`). Run `node --test tests/stdio-conformance-runner.test.mjs` to validate the runner's guardrails; this **does not run a guest**. Real per-backend conformance requires an explicit compiler/runtime configuration and exact pinned Git heads. All eleven backends remain `NOT_EXECUTED` until independently verified target-specific source-compiled execution or an explicit HDL unsupported-profile receipt exists.

## Canonical Atto source backend inventory (migration-safe)

The separate source-free inventory at contracts/v1/atto-backend-inventory-v1.json pins all eleven Atto compiler repositories to their numeric GitHub repository IDs, 2026-11-21 main commit/tree SHA pairs, branch protection and sampled CI admission status. It is an immutable historical snapshot, not a live compliance decision or a private-source transfer grant.

Run node --test tests/atto-backend-inventory.test.mjs to deny missing/renamed backends, repo-ID spoofing, fabricated guest passes and zero-step CI impersonation. The older manifest.json and contracts/attolang-semantic-floor-v1/contract.json include migration-era Oreslang sources; they are not automatically a canonical Atto roster. The new test cross-checks PR #23's eleven-backend stdio/ADT matrix.

All eleven source main branches were unprotected at the time of this audit. Ten backends' sampled latest CI jobs were zero-step and runner-unassigned (including the older Atto self-hosted run 44175114352), while no recent workflow was found for realtime. No compiled Atto guest parity was verified by these metadata checks. Future guest claims require identical .atto fixture blobs, source/toolchain identity, real executed test steps, nonzero runner identity, observed guest output, independent review and source export authorization.
