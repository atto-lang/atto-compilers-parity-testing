# Annotated mutable state — cross-backend candidate (v0)

**DRAFT / NOT EXECUTED.** This stacked candidate builds on [state-container PR #12](https://github.com/ores-truffle-oreslang/oreslang-compilers-parity-testing/pull/12) and intentionally does **not** modify the required `manifest.json` or its current 5-backend matrix. None of these fixtures establishes backend compilation, heap allocation, race freedom, or guest execution.

## Canonical proposal

`@Inject(MutableStateContainer())` on a `define class` with a `state { ... }` declaration is a **proposed compiler-recognized storage injection**, not an ordinary dependency-injection constructor and not a claim that any currently released backend implements that syntax.

- Desugar to a **private, non-escapable state region**. `self.handle_state(critical[read] |s| -> { ... })` provides scoped immutable access; `critical[write]` provides scoped exclusive mutation. Both callbacks are **synchronous and void**. Exactly one logical region authority exists per instance; inheritance cannot duplicate or expose it.
- Static borrowing applies to **allocation identities and projected places** regardless of stack/heap placement. A copied reference is not a copied owner. `rt take` revokes its former owner's authority. Read/write guards and closures must not escape, be promoted to a heap-stored alias or remain live over `await`/`select`.
- Injection itself never implies `Sendable`, thread-safety, shared mutation or cross-isolate access. Raw `rt mut share` of this mutable class must fail. Any later `SharedMutex<T>` admission requires proof of a synchronized handle, linearization, cancellation-safe release, reentrancy and retirement identity. A private region cannot be copied into an unguarded shared heap.
- Local uniquely owned heap mutation may need **no runtime mutex**. True shared heap state requires an authenticated guard and safe publication. Runtime guards complement, not bypass, static ownership checks.
- Constructor/teardown must initialize and retire exactly once; rejected or failing operations cannot leave partial updates; unchecked pointer/FFI routes require separate safety proof.
- For actor injection, preserve sealed actor-state semantics and runtime-minted physical state loans; do not replace the actor `handle_state` gate with a host proxy. Class state and actor state may share the type-level contract but not fabricate actor execution IDs.

## Candidate source oracles

`manifest.json` pins seven **proposed** exact source cases and the **five requested backend names**: Java/Graal, LLVM, WASM, Cranelift, and .NET Native AOT. **Forge is a distinct backend from Cranelift**, and JS remains in the existing required runner rather than this candidate.

The positive owned counter should print exactly `2`, exit 0, and remain invariant-safe. The six negative programs target private field bypass, mutation from read loans, escaping a state loan, suspension with a live loan, use after take, and unguarded raw mutable sharing.

A negative counts only after the same source has passed parsing of the candidate feature and the compiler rejects with the specified **type/ownership diagnostic category**. Unsupported syntax, parser errors, missing compilers or a generic nonzero exit do **not** count as a semantic rejection. A compiler that rejects all seven programs has zero demonstrated passes.

## Required sequence before promotion

1. Review an annotation grammar/AST shape and generated-field ownership semantics in `oreslang-common-source`; settle exact keywords and the `critical[read]`/`critical[write]` form. Verify no collision with existing annotation expansion.
2. Implement Java compiler admission, ownership and runtime guard semantics with real positive/negative JUnit guest fixtures. Preserve original source bytes.
3. Port the **same source** to LLVM and Cranelift with deterministic region lifetime; then .NET Native AOT and WASM worker/runtime lowering. Reject unsupported capabilities explicitly. LLVM zero tracing-GC and Java minimal-allocation claims require actual IR/heap and allocation measurements.
4. Run exact-head compiler commands and actual produced guest applications on each backend, plus ASan/UBSan where applicable, race tests and cancellation/ABA guard tests. Record emitted artifacts, SHA, nonzero executed CI steps and diagnostic categories.
5. Integrate with a reviewed Rust adapter before marking candidate validated or adding to the required manifest. No source rewriting or private-org mirroring without approval.

Related work: [Java compiler baseline #2317](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2317), [Java guarded mutable sharing #2385](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2385), [Java runtime state-loan hardening #2363](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2363), [LLVM native borrow #1194](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1194), [WASM provenance #187](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/pull/187), and [.NET provenance #261](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/261).

**Verification:** authoring/source contract only. No five-toolchain execution or parity result is claimed.
