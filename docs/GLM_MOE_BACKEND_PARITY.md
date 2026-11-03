# GLM/MoE five-toolchain parity acceptance — 2026-11-01

**Status: authored shared-source candidate corpus; no five-toolchain parity proven.**

The branch registers twenty *identical* Oreslang source programs at
`programs/32-glm-*.ores` through `programs/51-glm-*.ores`.
Every case is required on Java/Graal, LLVM, WASM and .NET. Each successful
guest returns integer 42; a semantic mismatch returns 1. None is marked
`validated` before trusted execution.

## Feature matrix and real gaps

| Feature | Java/Graal | LLVM native | WASM (LLVM → Wasmtime) | .NET NativeAOT |
|---|---|---|---|---|
| Deterministic signed score top-1 | Execute 32–34 | Execute 32–34 | Execute 32–34 | Execute 32–34 |
| Capacity/exhaustion/assignment-count | Execute 35–43 | Execute 35–43 | Execute 35–43 | Execute 35–43 |
| Generation/cancellation/step order | Execute 44–48 | Execute 44–48 | Execute 44–48 | Execute 44–48 |
| Binary-fraction float prediction | Execute 49 | Execute 49 | **Known source float frontier** | **Known source float frontier** |
| Fixed-gate analytic derivative | Execute 50 | Execute 50 | **Known source float frontier** | **Known source float frontier** |
| Strict SGD loss reduction | Execute 51 | Execute 51 | **Known source float frontier** | **Known source float frontier** |

`Execute` is a **future acceptance requirement**, *not* a verified pass.
The Wasm `contracts/v1/numeric-parity/backend.json` currently declares
`binary32_arithmetic=unsupported_explicit`; the Wasm compiler bridge
uses an externally pinned native LLVM compiler and isolates Wasm guest bytes.
The .NET compiler currently has checked-i64 OresIR/CIL verification and a
NativeAOT host pipeline, but that evidence does not establish `float`
source lowering or GLM parity. Java/Graal, LLVM and Cranelift are also **unverified
on the new exact source tree** until genuine guest runs occur.

## Promotion procedure

1. Independent reviewer approves compiler/runtime revision, trusted runner
   admission, the source tree and backend-specific launch commands.
2. For each of 20 source files, record exact bytes/SHA-256, compiler SHA,
   runtime SHA, compile output, guest termination status and integer result.
3. LLVM: compile native at O0 and O2, run both resulting binaries, and
   attribute optimization-specific differences correctly.
4. Wasm: independently pin emitted Wasm SHA and Wasmtime execution,
   preserving explicit unsupported-source classification.
5. .NET: verify parsed source and OresIR, emit CIL, publish and **execute**
   a real NativeAOT host. No interpreter/IR-only substitute is accepted.
6. Reject absent compiler, unclassified diagnostic, timeout, missing guest,
   wrong return, unsanctioned source rewrite, or CI `steps=null`.
7. Keep all 80 required outcomes pending until separately executed and
   reviewed. Exact-head stepful CI is distinct from native guest receipts.

## What this corpus cannot prove

- The integer load-count programs do not establish *unique* token/expert
  assignments or atomic capacity reservations. See
  [Models issue #16](https://github.com/ores-llm/ores-llm-models.ores/issues/16).
- Step-order fixtures do not prove actual concurrent actor cancellation,
  persistent checkpoints, and generation-fenced ownership. An input-admitted
  SGD update can still exceed post-update bounds. See
  [Trainer issue #13](https://github.com/ores-llm/ores-llm-trainer.ores/issues/13).
- Fixed-gate scalar arithmetic is not a tensor MoE, sparse top-k backward,
  trainable gate, causal Transformer, optimizer checkpoint or trained GLM.
- Corpus size, source-only mathematical oracles and GitHub jobs with zero
  executed steps never count as compiler-backed execution.

## Backend sources examined

- `ores-truffle-oreslang/oreslang-source.java`
- `ores-truffle-oreslang/oreslang-source.llvm` and
  `scripts/compile-aot.sh`
- `ores-truffle-oreslang/oreslang-source.wasm` and
  `contracts/v1/numeric-parity/backend.json`
- `ores-truffle-oreslang/oreslang-source.net` and
  `verification/Ores.Net.Tests/SharedScalarParityConformance.cs`
- Shared failure evidence: 
  [OresLLM native guest gate #16](https://github.com/ores-llm/ores-llm-ml-conformance-tests/issues/16)


## Initial implementation audit — not native execution

The 20 programs were independently checked by a **restricted host-side
JavaScript surrogate** that translated their scalar function/if syntax into
JavaScript and evaluated each numerical control. All **20 of 20** produced
the required result `42`. This validates the *expected control arithmetic*
only; it does **not** validate Oreslang grammar, emitted LLVM/Wasm/CIL,
Truffle guest execution, NativeAOT linking, or numerical backends.

The checked manifest contains 51 legacy cases and 16 positive whole-program
checks plus 2 negative probes: **69 logical cases** and **345 five-backend
cells**. All new 20 GLM manifest rows remain `candidate`, explicitly
required on the five requested toolchains. The corpus now contains
17 integer-control and 3 floating-source GLM files.

The initial [Corpus audit run](https://github.com/ores-truffle-oreslang/oreslang-compilers-parity-testing/actions/runs/41343563533)
failed with `steps=null`, so runner admission/execution proof is absent.
Do not misclassify this as a Rust test, compiler or native guest failure.
There is no approved compiler-backed result for the new exact tree.

Tracked native acceptance by compiler:
- [Java/Graal #2421](https://github.com/ores-truffle-oreslang/oreslang-source.java/issues/2421)
- [LLVM #1276](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/issues/1276)
- [Wasm #244](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/issues/244)
- [.NET NativeAOT #307](https://github.com/ores-truffle-oreslang/oreslang-source.net/issues/307)

These are tracked implementation tasks, not passing evidence.

### Cranelift / fifth compiler target

Cranelift JIT must compile and execute the **identical 20 source files** under the sixth corpus adapter (`ORES_PARITY_CRANELIFT_COMPILE` and `ORES_PARITY_CRANELIFT_RUN`). A missing adapter, a checked-only result, or LLVM/Wasm execution claimed on behalf of Cranelift is **not** a pass. Validate the same 17 integer and three float programs against pinned compiler/runtime provenance, and record honest unsupported diagnostics until the frontend and lowering support them.


## Cranelift exact current capability and native-gate PR

The independently examined [Cranelift repository](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift)
is a **distinct native JIT**, not Wasmtime's LLVM substitute. Its current
`main` supports only one `main` function with 0–4 signed i64 parameters,
checked `+`, `-`, `*`, unary operators, checked signed remainder with a
literal divisor and a narrowly supported `Integer.bitXor` function.
It *rejects* user helper functions, Boolean control flow, imports, float,
actors, ownership and ordinary GLM/MoE modules.

[Cranelift draft PR #69](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift/pull/69)
adds 11 integration tests that must run through the actual native JIT for
the arithmetic intersection, and must fail closed for GLM syntax outside
that supported subset. **A deliberate reject is an honest capability
result but not a parity pass.** The current native CI has not established
a stepful exact-head execution receipt. Extending Cranelift to all twenty
unchanged GLM source files requires real parser/IR/CLIF lowering, not
removing helpers, replacing float with int, or claiming the Rust host oracle.

Cranelift's source tree snapshot is recorded in `manifest.json` as a
**snapshot-only** reference, never as native compilation evidence.


## Security and audit hardening — 2026-11-03

This change is **source-level audit hardening**, not compiler-backed guest
conformance. Key protections added on the shared draft branch:

- **Exact source revision:** the Corpus audit workflow checks out the actual
  pull-request head SHA instead of GitHub's synthetic merge ref, verifies
  `git rev-parse HEAD` against the expected revision, requires a clean
  working tree, and prints the immutable source Git tree identity.
- **Independent static matrix audit:** all 414 records must be unique
  backend/case pairs, with exactly 69 registered logical cases and six
  independently named targets. The workflow independently SHA-256 hashes
  all canonical files and positive reachable import graphs; a missing,
  duplicated, rewritten or fabricated `not_run` record is rejected.
- **Fail-closed source admission:** the Rust runner reads source and imported
  text through descriptor-level nonblocking/nofollow admission (on Unix)
  with a 4 MiB source budget, checked UTF-8, bounded SHA-256 reads and
  size-change detection. Source symlink, unbounded content, and invalid
  UTF-8 negative regressions are authored.
- **Cranelift frontier:** the independent [PR #69](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift/pull/69)
  now drains native compiler stdout/stderr concurrently with a per-stream
  64 KiB limit, rejects timeouts, and checks genuine frontend-diagnostic
  exit codes rather than accepting any nonzero process failure. It also
  adds oversized-output/source cases.

**Evidence restriction:** GitHub-hosted attempts on these exact heads still
end before step records are populated (`steps=null`). Neither Rust unit
tests, workflow checks, nor true Java/LLVM/Wasmtime/NativeAOT/Cranelift
guests have a newly verified stepful receipt from these commits. CI
execution, independent review, and native positive + negative cases are
required before any draft is promoted or merged.

**AttoLang migration boundary:** this historical compatibility corpus still
uses `.ores` source filenames. A separate exact-source `.atto` conformance
gate is needed for the public AttoLang naming transition across all
compilers; none of the current `.ores` receipts establish `.atto`
frontend/file-import parity. Do not silently rename or rewrite fixtures
behind backend adapters.
