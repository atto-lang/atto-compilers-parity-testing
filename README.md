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
