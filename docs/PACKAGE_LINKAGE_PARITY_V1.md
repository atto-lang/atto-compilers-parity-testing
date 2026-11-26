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
