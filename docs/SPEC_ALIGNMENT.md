# Spec-to-parity traceability

This is the source-only link between the [AttoLang language specification](https://github.com/ores-truffle-oreslang/oreslang-language-specs.md) and this repository's existing compiler corpus. The authoritative rules live in `spec/conformance.md` and `spec/parity-traceability.md` of the spec repository.

## Evidence boundary

`contracts/spec-alignment-v1.json` pins an immutable reviewed spec commit and edition, records six **independent** required backend IDs (`llvm`, `java`, `wasm`, `dotnet`, `forge`, `cranelift`), keeps `js` optional, and links proposed spec acceptance IDs to existing local fixture IDs.

Every link has `relation: related_feature_only`. A link is a review lead, **not** proof of full semantic coverage, corpus validation or backend execution. In particular, Forge cannot stand in for Cranelift and vice versa; missing adapter support remains an explicit gap. The Rust source-only integrity test checks that each linked case exists and that the six required IDs match the live corpus manifest. It does not compile a guest.

## Source-extension migration

New AttoLang language source and added conformance fixtures should be `.atto`. The published baseline corpus and its pinned hashes remain `.ores` as an explicit **legacy compatibility** profile. Never mutate an existing `.ores` fixture or change its extension just to appear compliant: that changes source identity and invalidates prior provenance. A migration should create a separately versioned `.atto` case, pin the exact source and import graph, and verify the intended oracle. The same exact bytes must be supplied to every required backend.

## How to extend coverage

1. Ratify or explicitly label the new language rule and stable spec fixture ID; update the spec catalogue and its edition.
2. Add a distinct positive/negative corpus fixture with a reviewed acceptance oracle and stable local ID. Add a `related_feature_only` mapping only when the relationship is truthful.
3. Run the Rust source integrity tests (`cargo test --locked --manifest-path tools/parity-runner/Cargo.toml`) and, separately, compiler plus guest execution with exact provenance per required backend.
4. Upgrade a conformance result only after executed diagnostic/runtime evidence passes the specific required contract. An inventory, source-only test, CI workflow trigger, or shared registry mapping never counts as a compiler parity pass.

Concurrent feature-expansion PRs modify `manifest.json` and Rust runner internals. This bridge intentionally does **not** change current fixture counts, oracle status, required-backend denominator or runner results; update its references against the final merged corpus when those PRs land.
