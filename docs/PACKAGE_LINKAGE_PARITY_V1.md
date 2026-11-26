# Package linkage parity rollout (unverified)
 
All eleven atto-source.* repositories are in scope. This file is a test plan, not a capability claim. Every executable guest cell initially NOT_EXECUTED. Static/dynamic/hybrid test results must come from actual compiler output, not hand-written C, JVM or Wasm host canaries.

| Repository | Required adapter | Explicit restrictions |
|---|---|---|
| atto-source.java | JVM package artifacts; Graal Native Image static/precompiled-native boundary | no arbitrary class loading under sealed AOT |
| atto-source.llvm | native object/archive/shared-library and ELF/Mach-O/PE inspection | current compile-aot.sh uses single-source IR and host linker; not verified incremental package emission |
| atto-source.net | CIL assemblies and NativeAOT guest/link boundary | managed assembly loading not arbitrary NativeAOT code loading |
| atto-source.rs | attoc source-to-Wasm and separate module interface | current linked compiler produces Wasm bytes, not proof of dynamic modules |
| atto-source.wasm | verified module instantiation and import/export validation | consume Rust source compiler receipts; not a second source compiler |
| atto-source.forge | AOT object and separate precompiled module admission | JIT is an independent execution mode |
| atto-source.cranelift | relocatable native object / module linker | no silently substituted JIT |
| atto-source.js | bundled output and ES module dependency graph | host module imports alone not proof of guest semantics |
| atto-source.hdl | static elaboration / synthesis | dynamic/hybrid software linkage explicitly unsupported |
| atto-source.atto | bootstrap compiler-owned package artifact | Stage-0 scalar evidence not self-hosting |
| atto-source.realtime | deterministic static link and timing admission | repository bootstrap; dynamic/hybrid hard-real-time unsupported in v1 |

## Scenarios
L01 cold dev static; L02 cold dev dynamic; L03 cold dev hybrid; L04 cold release static; L05 cold release dynamic; L06 cold release hybrid. All must execute same positive corpus and emit equivalent results. Inspect package linkage rather than system runtime linkage.
L07 no-op warm zero recompilation; L08 compatible body edit (static relink, dynamic immutable new module, hybrid selected boundary); L09 unrelated namespace object reused; L10 public ABI change rejects stale client; L11 changed ownership/lifetime/effect contract; L12 generic inline-body change; L13 actor mailbox/state layout change; L14 same package name distinct pinned versions; L15 package lock / alias rebind; L16 missing or removed exported symbol; L17 tampered artifact; L18 target/runtime ABI mismatch; L19 denied capability before initializer; L20 failed compile/link preserves last success; L21 cancellation before publication; L22 old generation retained during suspended call; L23 late-generation callback denied; L24 incompatible reload rolls back; L25 sealed AOT rejects dynamic and JIT fallback; L26 unsupported target/mode explicitly diagnosed; L27 deterministic clean-vs-warm diagnostics and behavior; L28 cache domain separation across link mode/target; L29 missing dependency must not fallback.

## Receipt requirements
Compiler commit and root tree, each fixture digest, target, compiler/runtime toolchain, optimization/link/execution profiles, actual emitted artifact hashes, link dependency inspection, compile/relink/reuse counters, executed command exits and outputs, negative diagnostics and no artifact publication, runner allocation and executed steps. Record PASSED/FAILED/UNSUPPORTED/NOT_EXECUTED independently for each scenario/profile/target. Missing evidence never maps to PASSED. Do not bypass source-export policy or branch protections. Shared host canaries are separate receipts.

## Direct multi-provider fixture (L30–L37)

Executable adapter target: two independently compiled provider units, each exposing `value(): int`, and a consumer that binds their exported functions as distinct aliases. For explicit build-table adapters, use `cart@1 -> cart` and `tax@1 -> tax`; these are fixture logical identities, not a new normative package-import syntax or a signed identity. Initial values 40 + 2 produce 42. Replace cart with 41 (same interface), retain the exact consumer artifact and tax artifact, and obtain 43 from a new bound consumer; on dynamic generation-capable targets the original bound consumer still returns 42.

| ID | Required behavior |
|---|---|
| L30 | Independently compiled providers with identical export names, explicitly distinct aliases, execute 42. |
| L31 | Reorder dependency inventory; canonical dependency-plan identity and consumer artifact remain unchanged. |
| L32 | Change one provider body; new consumer binding executes 43; unrelated provider and consumer artifacts unchanged. |
| L33 | Duplicate dependency identity or alias, missing/extra provider, and unknown export fail closed. |
| L34 | Provider ABI and logical identity changes invalidate the dependency plan; reject stale consumer. |
| L35 | Tamper second provider bytes; all declared providers must pass admission before any trusted initializer executes. |
| L36 | Wrong imported parameter signature rejected even with recomputed candidate manifest digest. |
| L37 | Consumer artifact corruption rejected before provider access. |

JS #159 implements a bounded direct-provider scalar adapter (wrapping-i64 provisional); 6 new synthetic ESM runtime tests passed locally, combined runtime suite 23/23; **actual independently compiled Atto guest tests NOT_EXECUTED**. No general transitive initialization, unload, actor upgrades or cryptographic package provenance established. For static-only targets L32 requires relink using reused consumer object, not live loading; L35 applies to artifact admission before executing a linked guest. For HDL use static elaboration and reject runtime-only features. Every applicable backend still requires actual compiler-backed evidence; do not copy JS host-only pass status into other cells.

## Transitive closure and bounded artifact retention (L38–L45)

| ID | Required behavior |
|---|---|
| L38 | Diamond dependencies resolve a single exact shared package slot in dependency-first order. |
| L39 | Reordered inventory and import edges produce identical canonical release locks. |
| L40 | Compatible leaf edit changes release lock without recompiling unaffected consumers; new dynamic binding observes update, old binding retains old version. |
| L41 | Self-cycle, indirect cycle, missing dependency, duplicate package slot and unreachable package reject before artifact evaluation. |
| L42 | Transitive ABI mismatch and forged imported signature reject before any initializer executes. |
| L43 | All artifact bytes and separately trusted exact release lock admitted before evaluation; stale lock and tampered leaf reject. |
| L44 | Bound package invocation preserves scalar arity and numeric boundary checks. |
| L45 | Cache byte-budget eviction and oversize non-retention preserve cold/warm semantic equivalence. |

JS #159: 7 planner and 4 runtime synthetic ESM cases executed locally, total 34/34 runtime/planner regressions. Exact leaf/body and intermediate artifact pins, no cryptographic registry provenance, namespace-specific initialization state or unload claim. Actual Atto compiler-backed transitive guests remain NOT_EXECUTED. Rust #521 adds retained payload byte budget (default 16 MiB, configurable maximum 64 MiB) and two compiler regression tests, NOT_EXECUTED until Cargo runner. This is a retention budget, not a bound on process memory. All eleven compiler adapters require independent source-backed execution; static-only targets prove relink/reuse instead of live replacement.
