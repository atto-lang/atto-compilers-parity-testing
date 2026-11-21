# Atto actor critical / Ref32 — all 11 compiler parity admission (2026-11-21)

**Status: HOST-ONLY ADMISSION VALIDATED; 0/11 COMPILER GUESTS CERTIFIED.**

The canonical three `.atto` sources live in `contracts/atto-actor-critical-v1/fixtures/`; their exact Git blobs are pinned in `all-eleven-backend-admission.json`. Do not rewrite them per backend. The source-only positive asserts two-actor **logical** isolation, not physical memory segregation; the two negatives require semantic rejection of send-from-critical-read and write-through-critical-read. A generic parse error or "unsupported" result is NOT a negative semantic pass.

The all-eleven registry and runner are **additive** to the historical six required Oreslang parity profiles. Existing 54 × 6 = **324** historical corpus cells remain the default; the opt-in inventory provides 54 × 11 = **594** possible inventory cells, not 594 guest passes. Run:

```sh
cargo run --manifest-path tools/parity-runner/Cargo.toml --locked -- --output default-six.json
cargo run --manifest-path tools/parity-runner/Cargo.toml --locked -- --all-backends --output all-eleven-inventory.json
cargo run --manifest-path tools/parity-runner/Cargo.toml --locked -- --backend rs --output rust-inventory.json
```

No `--execute` means inventory-only `not_run`; the runner's `--execute` gate fails when adapters, source language, semantic diagnostics or receipts are missing. **Do not promote results from Oreslang `.ores` compatibility corpus to canonical Atto `.atto` actor certification.** A separate source/guest runner must exercise this exact three-file contract, with no backend-specific modification and exact compiler binary SHA. Never use Python for the Rust-only parity runner.

## Backend admission snapshot

These are **audited GitHub default-branch identities**, not proof of guest support. All 11 rows remain `NOT_EXECUTED` for the canonical actor-critical guest. An independent compiler/runtime implementer must refresh the SHA and provide stepful evidence.

| Backend | Exact main commit (2026-11-21 snapshot) | Integration priority / current limitation |
| --- | --- | --- |
| Java / GraalVM | `12ded133d89dafb21f48391e9aa3d5a7fb7f80f6` | [Sealed physical state loans](https://github.com/atto-lang/atto-source.java/issues/404); compiler-emitted guest + JDK25 Native Image required |
| LLVM | `fe4f253b5932f8f00ab4a20f9b9bc2019d17553b` | [Compiled sealed-state loan proof](https://github.com/atto-lang/atto-source.llvm/issues/222), [Ref32 bounds PR #823](https://github.com/atto-lang/atto-source.llvm/pull/823); O0/O2 + sanitizers required |
| .NET | `5ed2444f3526b8b6685c7be0b038bbb8e6bc6f51` | [Real arena/Ref32 lowering](https://github.com/atto-lang/atto-source.net/issues/93), exact issued capability [PR #143](https://github.com/atto-lang/atto-source.net/pull/143); NativeAOT (not CoreCLR substitute) |
| Rust compiler | `7f893323f761e4a71e4b2e199daf75845d8f1912` | [Field-loan PR #476](https://github.com/atto-lang/atto-source.rs/pull/476); execute guest compiler separately from canonical Rust host-model tests |
| Wasm | `16ba74ab9f9959cc15d21dcb6e54a18e1c325fef` | [Critical/async rejection PR #175](https://github.com/atto-lang/atto-source.wasm/pull/175); actual wasm32 guest, memory/generation constraints |
| Forge | `f5dd299c277376cabe9e536edbbb4e662a267ad4` | [Unsupported critical/suspends refusal PR #157](https://github.com/atto-lang/atto-source.forge/pull/157); no interpreter-to-native-AOT substitution |
| Cranelift | `e867c0a5cf8b94f98ce9b1212d2c735b76bc4120` | Stage-0 scalar JIT, critical source currently rejected; actual actor lowering not implemented |
| JavaScript | `b8e48b19c33b61e5faa4628bf03bddf2823f403d` | [Worker guarded-state preview PR #42](https://github.com/atto-lang/atto-source.js/pull/42); trusted worker provenance and executable guest semantics missing |
| HDL | `023455e496bb9cfbdd157ccc94bb69f31eec44ea` | Synthesizable subset; dynamic actor/heap/critical semantics require explicit unsupported status unless truly lowered to hardware |
| Realtime | `4f6449470ab075cdad17884c7a1aee2a6e2ab54c` | **Repository bootstrap only**, just README and AGENTS; no real-time compiler or hard-RT guarantees |
| Self-hosted Atto | `5b9a274490a4567ed9d1088fd1ff304dbe383357` | **Not self-hosted**, just README and AGENTS; requires stage-0 bootstrap, compiler rebuild, self-hosted guest proof |

## Implementation acceptance gates

1. Audit competing PRs and independent local borrow checking; every compiler's typed MIR must contain canonical actor, arena, incarnation, allocation ID, allocation generation and complete effect inventory. If unsupported, reject before codegen; never trust guest-created proof flags.
2. Validate stable source-to-MIR identity, then derive immutable compiler evidence; bind to **runtime-minted** actor/arena/allocation identities and sealed physical turn/loan capabilities. Numeric actor IDs and equal offsets are not authentication.
3. Keep a real reader/writer guard across each native/wasm/CIL/JS guest load/store. STOP, cancellation, panic, task handoff and hot reload must fence current and queued permits; physical quiescence precedes memory reclamation.
4. Require exact identical original positive and two negative `.atto` blobs per backend, distinct targeted static diagnostics (not generic parser rejection), actual guest execution stdout, no unexpected artifact for negative cases, executable compiler/toolchain hash, source tree and nonzero CI steps.
5. Native/AOT/sanitizer/GC/quotas/actor-isolation tests are independent proof levels. Missing and unsupported backend cases remain `NOT_EXECUTED` or explicit unsupported gaps, NEVER `PASS`.

## Host-only receipts; NOT guest executions

- [Canonical Rust borrow-checker #29 model carrier run 44939532568](https://github.com/ores-data-platform/atto-ci/actions/runs/44939532568): exact-tree host Rust 222 debug + 222 release tests. This **cannot** grant native memory authority.
- [Eleven-compiler registry carrier run 44945971941](https://github.com/ores-data-platform/atto-ci/actions/runs/44945971941): 5/5 registry tests + strict Clippy, SHA-verified subset.
- [Eleven-selector runner carrier run 44946572354](https://github.com/ores-data-platform/atto-ci/actions/runs/44946572354): 5/5 registry + 11/11 binary unit tests + strict Clippy and formatting, source-bounded private carrier.
- Source [parity runner CI admission issue #25](https://github.com/atto-lang/atto-compilers-parity-testing/issues/25) remains open. Source Actions can fail with `runner_id=0` and `steps=[]`; that is not evidence of executed tests or a code regression.

This ledger is updated through the implementation PR, with no change to existing three-backend fixture bytes. Do not mark the draft Ready, merge or claim certified parity until independently executed source CI and review requirements are met.
