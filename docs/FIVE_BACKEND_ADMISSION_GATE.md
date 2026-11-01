# Java / LLVM / Wasm / Cranelift / .NET parity admission gate

Status: **draft source/test hardening, not five-backend native parity** (2026-11-01).
The shared runner also inventories JavaScript; that additional backend must never
silently disappear from the canonical six-backend policy.

## The shared scalar boundary

| Contract | Java/GraalVM | LLVM | Wasm | Cranelift | .NET Native AOT |
| --- | --- | --- | --- | --- | --- |
| Signed i64 literals/arithmetic, precise boundaries, checked overflow | #2405 and scalar reference fixtures | #1256 and #1252 | #218/#219/#222 (pinned LLVM delegation) | #22 JIT corpus, experimental only | #282/#285 typed CFG/CIL |
| Inferred immutable scalar bindings | #2405 source and guest tests authored | #1256 native scalar slice | #219 delegated compiler | #21 provisional i64 local subset; no general inference | #282 typed scalar subset |
| Checked identity cast to `int` / `bool` | #2405 tests | #1256 constrained lowering | #219 compiler-to-Wasm gate | **Not lowered**, reject in #22 | #282 typed scalar subset |
| Checked optional/dynamic/nominal cast | Feature-dependent; prove each concretely | Not lowered; reject | Not lowered; reject | Not lowered; reject | Not lowered; reject |
| Actor isolation, ownership, async/await, imports | Independent compiler/runtime proofs required | Independent zero-GC native proof required | Worker+Wasm isolation proof required | Not supported in scalar JIT | AOT tenant/isolate and runtime proof required |

The PR numbers above are **authored changes**, not verified exact-head execution
receipts. Cranelift is not an LLVM alias, and Wasm's current LLVM compiler
delegation does not constitute an independent compiler proof.

## New harness acceptance invariants

1. The canonical manifest must list the exact six runner backend IDs (including
   JavaScript) exactly once. Missing or duplicate IDs abort inventory and execution.
2. Failed compiler admissions are classified using **stderr only**, never
   source-controlled stdout. Nonzero exit codes and diagnostic text alone are
   insufficient: a negative admission must leave the isolated build workdir
   empty. This includes partial guest binaries or published links.
3. Every requested `--execute` operation is a **strict** gate: only
   `pass` or `expected_reject` counts as success. `unvalidated`,
   `exploratory_*`, `backend_not_configured`, timeout, missing adapters,
   unsupported capabilities and unknown future statuses fail the process.
   The default no-`--execute` inventory remains nonexecuting and may exit zero.
4. Duplicate or unknown CLI switches are rejected; selecting multiple different
   backends cannot silently use the first.
5. A pass-shaped adapter receipt without provenance is not native evidence.
   Independently pin compiler and runtime source commits, toolchain versions,
   executable digests, input source graph SHA-256, output artifact digests,
   exact branch head and non-skipped guest execution. Review negative logs and
   empty-on-reject artifact traces.

## Smallest promotion sequence

1. **Admission:** resolve runner/billing/private policy and require a real
   workflow job with **nonzero executed steps**. A zero-step failed run means
   `NOT_EXECUTED`, not a source or compiler failure.
2. **Harness:** on the exact parity PR head execute
   `cargo test --locked --manifest-path tools/parity-runner/Cargo.toml`,
   `cargo clippy --locked --all-targets --manifest-path tools/parity-runner/Cargo.toml -- -D warnings`,
   and a no-execute 49 x 6 manifest inventory. Assert all new CLI and Rust
   regression cases actually ran.
3. **Native scalar:** build exact-head Cranelift, LLVM, Java and .NET compiler
   artifacts. Run Cranelift's native JIT positives/negatives, LLVM's compiled
   native signed-i64 and cast cases, Java's pinned JUnit+guest cases, and
   .NET's CFG/CIL verifier, then a **separate** NativeAOT executable lane.
4. **Wasm:** verify the exact LLVM source and compiler digest, emit wasm32,
   check input/wasm digest and import/capability policy, run the Wasm guest,
   and require zero publication on compiler-negative sources.
5. **Cross-backend:** compare **byte-identical** Oreslang input fixtures,
   expected integer values, diagnostics, exit status, and artifact outcomes.
   Report explicit `GAP` rather than counting a backend that rejects an
   unsupported language feature as equal to a native successful guest.
6. **Security/review:** independent compiler/ownership review and source
   confidentiality sign-off before any promotion. Do **not** transfer private
   Oreslang source to another organization to work around failed hosted jobs;
   security issue [platform-conformance #22](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/22)
   is still open.

## Related draft work

- [Java scalar inference/casts #2405](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2405)
- [LLVM scalar inference/casts #1256](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1256)
- [Wasm pinned scalar inference/casts #219](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/pull/219)
- [.NET typed CFG/CIL inference/casts #282](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/282)
- [Cranelift native signed-i64 scalar vectors #22](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift/pull/22)
- [Cranelift experimental immutable i64 locals #21](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift/pull/21)

Do not independently merge overlapping corpus additions in
[parity #16](https://github.com/ores-truffle-oreslang/oreslang-compilers-parity-testing/pull/16)
without reconciling the shared manifest, backend ordering and counts.
