# Oreslang compiler parity testing

Shared source-level conformance corpus for Java/GraalVM, LLVM, WebAssembly, .NET and JavaScript.

**Current status:** 30 candidate programs, 150 backend/case cells, zero validated parity claims. The harness fails closed: unvalidated syntax, unsupported features, missing adapters and infrastructure failures cannot become passes.

Tooling follows the Oreslang organization Rust-only policy. See `docs/EXECUTION.md`, `manifest.json`, and `programs/`.
