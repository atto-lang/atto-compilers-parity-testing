# GLM/MoE four-backend parity acceptance — 2026-11-01

**Status: authored shared-source candidate corpus; no four-backend parity proven.**

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
source lowering or GLM parity. Java/Graal and LLVM are also **unverified
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
required on the four requested toolchains. The corpus now contains
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
