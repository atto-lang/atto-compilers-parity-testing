# Candidate: private owned mutable state, scoped injected authority (v0)

**Status: candidate only / not_run.** Neither source is included in the required
`manifest.json`, so adding this directory cannot produce a false parity pass
or alter existing required backend/case counts.

## Exact unchanged-source oracles

- `owned-inventory.ores`: same content bytes as the Java draft source
  `oreslang-source.java` PR #2296's `examples/state-container-injection.ores`
  (source blob `505a98aba446206cd5da5f04878bf65e0c8fc992`).
  Expected stdout: `7|3|true|false|false|false` with successful process exit.
  A single exclusively owned class uses **private** mutable fields, validates
  before any of two related writes, offers scalar-valued public reads and
  accepts injected `borrow`/`borrow mut` authority. A denied over-capacity,
  zero or negative request may not partially update the state.
- `private-field-bypass-reject.ores`: compile-time **access violation** when
  an external borrower reads a private mutable field, even though the class
  exposes a lawful public read method. Requires a type/visibility diagnostic,
  not parser failure, and no emitted executable/IR.

All backends must receive exactly these source bytes; no frontend-specific
syntax rewrite or surrogate is a parity pass. Source-level positive/negative
controls are paired to prevent unconditional rejection from looking safe.

## Backend admission gates

| Backend | Admission required before a PASS | Current evidence |
| --- | --- | --- |
| Java/GraalVM | JDK25 front-end denial + exact guest stdout | not_run at new head (#2296) |
| LLVM | exact source admitted, native guest stdout, private-field denial and IR inspection | not_run; narrower borrowing/Clang fixtures in #1194 are not full parity |
| .NET NativeAOT | identical parser/ownership and AOT guest stdout | not_run |
| Wasm | identical parser/ownership and executed guest stdout | not_run |
| Forge | identical parser/ownership and executed guest stdout | not_run |
| JS optional | same bytes and semantics; explicit unsupported otherwise | not_run |

A backend may report `unsupported` with a feature-specific diagnostic; it
must **not** be counted as PASS until the full source compiles and executes.

## Memory and safety acceptance

1. Exclusive storage may mutate without a lock. An injected read capability
   cannot mutate; an injected mutable capability cannot outlive its lexical loan.
   No escaped alias to the private mutable representation.
2. The two-field invariant is restored before each synchronous operation
   returns. This is **not** a promise of concurrent linearizability.
3. For LLVM, inspect emitted IR and native allocation/reclamation for truly
   zero tracing GC with deterministic arena/owned value teardown. A token
   record should erase in compiled operations where possible.
4. For Java/Graal, measure allocations per operation after warmup on a supported
   runtime; do not infer zero allocations from source syntax or Native Image.
5. When extended to shared state, require nonparking FIFO/reader-writer
   admission, generation-fenced task authority, checked lock ordering,
   cancellation safety and actor retirement/quiescence. Reentrant actor turns
   remain forbidden until an explicit safe admission policy exists.
6. Tests need exact source SHA, nonempty runner/job steps, compiler/build
   evidence and guest execution outputs. Infrastructure-only success or a
   source-free canary is not source compilation evidence.

Related active drafts: [Java #2296](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2296),
[LLVM local/borrow #1194](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1194)
and [LLVM task-bound region #1192](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1192).

This is a same-organization, small source-level conformance candidate, **not**
a private-source tree/branch mirror to a CI carrier. No alternate-organization
private-source export is authorized by this artifact.
