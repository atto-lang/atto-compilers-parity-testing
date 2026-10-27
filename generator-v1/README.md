# Generator v1 parity corpus

This directory is an additive generator-specific corpus. It does not change the root parity manifest; the select-delimiter corpus and this generator corpus remain additive.

Every file is supplied byte-for-byte to Java/Graal, LLVM, WebAssembly, .NET Native AOT, and JavaScript. Unsupported features and infrastructure failures are parity gaps, not passes.

Current cases cover canonical bare sync `gen*`, `async gen*`, `suspend gen*`, sync/async iterator protocol mismatch diagnostics, static class generators, the fail-closed receiver-lifetime gate for instance generator methods, and compatibility spellings retained during migration. Additional lifecycle fixtures should be added only with a deterministic oracle for cleanup, concurrency, or shutdown behavior.

The canonical semantics are defined by `oreslang-common-source/docs/GENERATORS_V1.md`.

Expected status values are `verified`, `compile_reject_expected`, `unsupported`, `failed`, and `not_executed`.

A source-repository job with no executed steps is `not_executed`. Private source may be copied to a different Actions carrier only after the source repository's private-mirroring security policy authorizes that exact destination and workflow.
