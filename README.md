# Oreslang compiler parity testing

Shared source-level conformance corpus for Java/GraalVM, LLVM, WebAssembly, .NET and JavaScript.

**Current status:** 30 legacy single-file candidates plus 16 multi-file checks and 2 explicit negative probes = 48 logical cases / 240 backend cells. There are still zero validated cross-backend parity claims. The harness fails closed: unvalidated syntax, unsupported features, missing adapters and infrastructure failures cannot become passes.

The multi-file corpus lives under \`checks/<lowercase-name>/\`. Each positive entry point imports three local source files, performs in-program semantic assertions and emits a stable \`PARITY_OK|<name>|\` prefix. The Rust runner recursively fingerprints the reachable import graph so a backend result is tied to the complete source graph rather than only \`main.ores\`.

Tooling follows the Oreslang organization Rust-only policy. See \`docs/EXECUTION.md\`, \`manifest.json\`, \`checks/manifest.json\`, and \`checks/README.md\`.
