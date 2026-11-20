# AttoLang semantic floor v1 — staged conformance lane

**State:** authored source-level contract, not executed compiler conformance.

The [canonical contract](../contracts/attolang-semantic-floor-v1/contract.json) and four `.atto` candidate sources have matching intended bytes in the [language-spec repository](https://github.com/ores-truffle-oreslang/attolang-language-spec.md/tree/main/contracts/attolang-semantic-floor-v1). See [language spec chapter 21](https://github.com/ores-truffle-oreslang/attolang-language-spec.md/blob/main/spec/21-semantic-floor-and-parity.md).

## Compatibility boundary

The existing 53-case, 318-cell inventory is a **legacy `.ores` corpus**. It remains unchanged, including historical file paths, outputs, diagnostic expectations and source graph fingerprints. This new lane is additive and is **not** part of those counts or of any passing feature status.

All new fixtures are `.atto`. A backend must admit the canonical extension under a documented resolver contract; renaming `.atto` to `.ores` for an adapter, evaluating a host-only substitute, or secretly changing source by backend is not equivalent execution.

## Admission and evidence

For each proposed promotion:

1. Freeze the feature rule, required backend set and positive/negative oracle; independently compare exact Git *blob IDs* for contract and fixtures across the two repos.
2. Validate compile-time rejection by diagnostic *category* and source span, not a generic parse error, timeout, ICE, tool absence or process crash.
3. Compile **the same fixture bytes** for Java, LLVM, Wasm, .NET NativeAOT, Forge and Cranelift. Record each compiler head/tree and the fixture blob. Prove that an admitted positive actually executes and equals its typed oracle.
4. Check the exact workflow blob, run ID **and run attempt**, runner identity, completed meaningful steps, artifact SHA-256 and executed guest result. Independently bind the receipt to the parity cell; `RUNNER_EXECUTED` and `HOST_TESTS_EXECUTED` alone are not enough.
5. Test negative mutations, source ownership, cancellation/actor races, and full AOT/no-interpreter-fallback claims where relevant. With no trusted receipt, use `NOT_EXECUTED`, not pass/skip.
6. Only after evidence is stepful and independently reviewed, add a **separate versioned `.atto` inventory** to the runner; never reuse case IDs or alter the existing 53-case baseline without a migration PR and explicit review.

The Rust integration test `tools/parity-runner/tests/semantic_floor.rs` checks contract structure, semantic feature IDs, file reachability, extension policy, and that the staged cases cannot claim execution. It does **not** invoke any compiler.
