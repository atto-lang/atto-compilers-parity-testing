# Generator v1 parity corpus

This directory is an additive generator-specific corpus. It does not change the existing 30-program manifest.

Every file is supplied byte-for-byte to Java/Graal, LLVM, WebAssembly, .NET Native AOT, JavaScript, and Forge. Unsupported features and infrastructure failures are parity gaps, not passes.

Current cases cover canonical bare sync `gen*`, `async gen*`, `suspend gen*`, sync/async iterator protocol mismatch diagnostics, static class generators, the fail-closed receiver-lifetime gate for instance generator methods, and compatibility spellings retained during migration.

### November 12, 2026: lifecycle wave (new targets, not verified)

- `g11`: synchronous `Iterator.close()` after one yield, followed by repeated DONE and empty terminal value. A second producer value (99) must never escape.
- `g12`: `await AsyncIterator.close()` after one yield, followed by repeated asynchronous DONE and empty terminal value. This checks the public Future-shaped API; physical retirement and exactly-once `finally` still need separate adversarial runtime witnesses.
- `g13`: `yield* inner()` preserves 20/22 order into `outer()` and returns 42.

Each new positive fixture uses a deterministic `exit:42` oracle; unexpectedly emitted data, invalid DONE, or protocol mismatch returns a different exit or fails. Compilation alone does not establish any generator runtime result.

**Important:** generator-V1 main support differs by backend. LLVM/.NET have bounded synchronous literal-yield subsets; that is not general `gen*`, `yield*`, or async support. Forge's source compiler still fails closed even though an isolated frame substrate exists. Java's canonical branch remains draft; full JVM, native AOT, and actor-proven continuation behavior are not verified. Do not update any backend to `verified` based on a fixture's presence.

Further concurrency, cancellation, cleanup, and shutdown fixtures require deterministic independently executed witnesses and should not be labeled supported based on source-free models or zero-step CI.

The canonical semantics are defined by `oreslang-common-source/docs/GENERATORS_V1.md`.

Expected status values are `verified`, `compile_reject_expected`, `unsupported`, `failed`, and `not_executed`.

A source-repository job with no executed steps is `not_executed`. Private source may be copied to a different Actions carrier only after the source repository's private-mirroring security policy authorizes that exact destination and workflow.
