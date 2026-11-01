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

- `private-field-write-reject.ores`: **compile-time private-field write violation**
  even when the caller holds a valid mutable borrow of the instance.
  This is a privacy/access-control denial, not an immutable-borrow denial.
  Any backend that compiles the attempted external mutation fails this oracle.
  The successful `available_count()` getter proves the instance exists and
  the source is not an unconditional parser-rejection test. No executable/IR
  must be emitted for this negative candidate.

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
| Cranelift | same source bytes in native Rust JIT, real guest stdout and specific unsupported-feature diagnosis | not_run; currently scalar subset with actor/ownership features fail-closed |
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

## Mutable-state reader/writer lifecycle gate (proposal, not executed)

The class-inventory fixtures above exercise **exclusive owner** mutation only.
They do not prove actor-shared state, a two-phase scheduler-slot reservation,
or safe reuse of execution authority after cancellation. To promote shared
reader/writer mutable state beyond the current Java draft, every required
backend must independently prove the same observable contract:

1. Two readers coexist; a queued writer excludes subsequent readers until
   existing readers retire; no writer/read overlap or writer starvation.
2. A pending cancellation CAS may win before its revoker/hook obtains the
   arbiter lock. Slot/execution authority remains fenced until the winning
   Future's **terminal notification drain**, not merely queue removal.
3. A granted loan may be released inside its first host completion observer;
   no rematerialized logical execution, reused raw scheduler slot, or newer
   generation can acquire until *all* terminal observers finish.
4. For shutdown-failed queued Futures, failure callbacks must drain before
   authority reuse or physical actor-arena retirement.
5. A cancellation revoker failure poisons the actor state arbiter, denies new
   read/write admissions and promotion of older queued requests, fails them
   with the preserved cause, and prevents safe-reclamation wakeups.
6. A released old loan cannot certify `readyToSuspend` while any newer loan
   reserves its execution ID or scheduler slot. A foreign or stale generation
   must never authorize suspension.
7. Cancellation and close are idempotent, and physical retirement is notified
   at most once after the final physical operation and notification drain.
8. Neither a runnable code-path that always rejects nor a synthetic host-only
   state machine counts as **guest parity**: run a positive and negative source
   program using the actual compiler/runtime and record exact head, nonzero
   steps, diagnostics, emitted code, output and toolchain identity.

Grounded Java authored/unexecuted regression corpus: `ActorStateRwLoansTest`
in [Java #2363](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2363),
stacked on Java #2296. LLVM's task-bound state-region groundwork is in
[LLVM #1192](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1192);
WASM #187, Forge #148 and .NET #261 share the byte-identical
`state-region-provenance-v1.json` nine-negative host receipt contract.
None proves this actor-shared lifecycle or guest execution across all five
backends yet. Do not update the required `manifest.json` or promote a
candidate without independently executed semantic and security gates.

**Backend distinction:** `oreslang-source.cranelift` is a separate Rust/JIT
backend from `oreslang-source.forge`. Current shared `manifest.json` targets
Java, LLVM, WASM, .NET and JS; it does **not** require Cranelift or Forge.
The extra candidate rows above add neither backend to the required count.
Cranelift currently has fail-closed actor admission in drafts such as
[Cranelift #25](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift/pull/25)
and native scalar coverage in [#22](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift/pull/22).
To count five requested toolchains (Java, LLVM, WASM, Cranelift, .NET), the
required manifest and Rust adapters must be explicitly updated and their real
compiled guest runs validated. An unsupported Cranelift actor source may pass
a *negative diagnostic test*, but must never count as executable actor parity.

## Incremental ownership and private-state candidate corpus (2026-11-01)

The four additional source fixtures in `ownership-and-private-methods-v1.json`
cover two mutable aliases, use-after-take, read-borrow to mutable receiver,
and a paired positive public-method-only source returning 7.
The positive case intentionally **does not** use `define state container`,
which is not yet an executable nominal kind. It isolates existing class/borrow
rules before staged compiler lowering.

**Evidence: not_run on every backend.** These are candidate sources, not
accepted compiler fixtures. Semantic rejection must happen at the intended
checker stage; an unrelated parser failure is not a passing rejection oracle.
The 53-case required parity manifest remains unchanged, and no source bytes
have been transferred to another organization. Cross-backend promotion
requires identical source SHA, real guest compilation and executable output
and nonzero executed CI steps, including Cranelift as a separate toolchain.
