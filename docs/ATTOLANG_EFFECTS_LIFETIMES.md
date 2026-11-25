# AttoLang effects and lifetimes — exact-blob candidate lane

**Source-only staged, NOT_EXECUTED.** Canonical draft: https://github.com/ores-truffle-oreslang/attolang-language-spec.md/pull/33

This mirror contains the exact contract and four `.atto` fixture blobs from the spec PR head as of creation. `blob-manifest.json` records Git blob SHA-1 for the contract and each fixture. The Rust integration test `effects_lifetimes.rs` checks identity, typed categories, immutable status, and that a one-byte mutation changes the blob identity. It does **not** compile AttoLang or establish lifetime/effect parity.

No legacy `.ores` case count or historical result has been promoted. Backend parity still requires the same bytes compiled/executed at exact heads across Java/GraalVM, LLVM, Wasm, .NET NativeAOT, Forge, and Cranelift, matching exact negative diagnostics and positive output, with executed CI steps.
