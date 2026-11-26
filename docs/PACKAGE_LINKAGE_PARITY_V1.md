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

## Persistent source-unit artifact store (L46–L53)

| ID | Required behavior |
|---|---|
| L46 | Cold compilation and warm authenticated artifact reuse emit byte-identical guest output. |
| L47 | Process restart reuses an independently pinned compiler artifact without compiler invocation. |
| L48 | Compiler closure, target, toolchain, source, semantic configuration or package identity change invalidates the cache key. |
| L49 | Authenticated receipt corruption, stale ledger epoch and wrong trusted key reject; no execution from unverified bytes. |
| L50 | Failed compilation and oversized artifact publication preserve last committed artifact. |
| L51 | Concurrent publishers expose only complete authenticated snapshots; contention never steals live staging. |
| L52 | Bounded committed/staging storage and explicit abandoned-stage recovery; reject symlink/hardlink/root replacement. |
| L53 | Independent namespace edits preserve unaffected namespace cache entries and generated artifacts; require actual per-unit compiler counters, not a whole-program cache hit. |

JS #154: executed 6/6 Node 22.16.0 real filesystem/cache tests on exact store, test and receipt blobs; two concurrent processes and 256 publications included. Trusted Linux directory, 16 direct-mapped committed slots and 16 fixed staging names (128 MiB plus envelopes), offline stale cleanup. Actual closed scalar source-unit compiler integration and process-restart test authored, NOT_EXECUTED; not yet true namespace-selective code generation. All applicable atto-source.* compiler adapters must prove L46–L53 independently. HDL/realtime require target-appropriate static artifact and determinism receipts. No host-only pass propagated to guest cells.

## Integrated emitter and persistent ABI-keyed package artifacts (L54–L57)

| ID | Required behavior |
|---|---|
| L54 | Full checked IR is validated before function-fragment reuse; cold/no-op/one-body-edit emission counters are 2/0, 0/2, 1/1 on the two-function fixture; emitted guest executes 42/43. |
| L55 | Imported slot boundary, target indices, public exports and provider signatures are reconstructed or invalidated correctly; failed edit preserves previous session; no externally supplied executable memo text. |
| L56 | Persistent consumer key uses namespace/source-unit slot, reviewed compiler closure, target/toolchain, source and canonical dependency ABI, not compatible provider implementation bytes. Actual source provider edit 42→43 reuses consumer bytes; ABI drift requires re-emission. |
| L57 | Explicit empty linkage mode is invalid, unlike omitted mode; reject before compiler invocation or output mutation. Actual toolchain final-link assertion and separate-object guest checks must execute. |

JS #159 integrates #154 with both parents preserved. Six local exact checked-IR emitter regression tests execute generated ESM and pass; constructed typed IR, NOT full source frontend. Persistent graph cache and full source consumer session tests authored, NOT_EXECUTED. Namespace slots are caller-selected scalar source-unit identities, not general Atto namespace lowering. LLVM #918 fixes explicit empty mode and corrupted off-LTO test assertion; local six admission-prefix cases pass; full native driver/compiler execution NOT_EXECUTED. All applicable backend cells need independent actual compiler evidence; no blanket parity claim.

## Metadata identity and native publication audit (L58–L60)

| ID | Required behavior |
|---|---|
| L58 | Structured package and namespace identities distinguish separator-bearing logical paths; strict source encoding rejects malformed input before cache reuse; canonical ABI digests ignore member ordering and change on interface or membership changes. |
| L59 | Atomic compiled artifact publication owns exclusively created staging, rejects directory and symlink/reparse destinations and preserves their targets; failed publication cannot overwrite prior output. Qualify platform-specific constraints. |
| L60 | Actual native final-link command carries the requested LTO flags; synthetic probe success alone cannot certify final link, and forced probe failure preserves previous output. |

Java #1887: 5/5 standalone JDK21 identity cases executed locally against exact source/test blobs, compiler integration and three JUnit regressions authored but not yet executed. .NET #280: checked exclusive-staging CIL publication and actual source-compiled tests authored, not executed locally (SDK absent); trusted parent assumption, not general special-file/hostile-filesystem or directory-durability proof. LLVM #918: exact repaired test script passes bash syntax; extracted wrapper executes real Clang/LLD off/full/thin and both forced failure paths (5/5); complete driver and actual compiler-backed tests not yet executed locally. Source CI for all three returned zero jobs.

New carrier capacity verified on pal-trace-test/atto-ci run 45754627008 and mean-muffin-test/atto-ci run 45754650640 (one allocated runner, five successful steps each). Public exact-head five-backend recovery submitted at pal-trace-test/atto-ci#2, head faa1912f06334f2283f3082143f12f0cd429be64. Public anonymous fetch and exact source commit/tree verification on runner, no private tree transfer. No source guest pass until actual job and test receipts are available.

## Restricted independent native module object milestone (L61–L63)

| ID | Required behavior |
|---|---|
| L61 | Independently compile two actual scalar source modules to separate native objects with distinct stable qualified symbols; verify physical ABI, object type, defined and undefined symbol inventory before publication. |
| L62 | Edit only one provider; relink with byte-identical second provider and consumer, compare edited and clean rebuild results, preserve old executable; measure actual artifact generation. |
| L63 | Wrong symbol/arity, malformed source, and unsafe destination reject without overwriting valid prior artifacts; no authenticated package or general namespace ABI claim based on host-only consumer. |

LLVM #918 full source AOT scope PASS at 863276be9a5ab0d88335cacac55557ff208ad6d2, tree 9d23e3a4d1f4af69920303ab45f513a4b705a225; mean-muffin-test/atto-ci run 45758417854 job 139702422845 runner 1000000067, 10 real native guest executions and 10 invalid/unsupported rejections, complete scoped IR auditor and host harness. New stacked LLVM #951 at 95a77323204525dc853ad676ed1ffc0b5e0ab2a9 implements restricted i64() module-object emission from actual compiler IR with native C consumer. Run 45762731599 job 139716270205 admitted; exact public source tree verified; final test outcome pending. This is an independent module provider stage, not general namespace resolution, language consumer code generation, authenticated persistent cache or other backend parity.

## Independent compiler-verified namespace consumers (L64–L70)

All eleven target adapters (java, llvm, net, rs, wasm, js, forge, cranelift, hdl, atto, realtime) must record per-scenario, per-profile execution evidence. These are acceptance requirements, not completed implementations.

| ID | Required behavior |
|---|---|
| L64 | Compile a real Atto namespace consumer against two independently generated compiler-verified public interfaces, with provider implementation sources absent; no hand-authored stub body or host-language consumer substitution. |
| L65 | Independently emit each provider and consumer target artifact; prove exact imported physical ABI and unique qualified bindings, then link and execute 42. |
| L66 | Cold build emits three artifacts; no-op emits none; body-only provider edit emits one and relinks to 43, with byte-identical other artifacts; compare with clean rebuild. |
| L67 | Changed export arity, type, calling convention, visibility, logical namespace or target ABI invalidates affected consumers; stale consumer and forged interface fail closed. |
| L68 | Missing, extra, duplicate, corrupted or wrong-target object/interface rejected before final execution; preserve prior artifact on failed admission. |
| L69 | Source alias (including hardlink), symlink, directory and FIFO destinations reject without mutation; relink from admitted artifacts after provider source removal. |
| L70 | Run O0/O2 or target-equivalent debug/release and record actual compiler invocation counters, artifact hashes, guest outputs, exact source tree and negative diagnostics; host-only checks and unsupported profiles remain separate. |

LLVM prior head 95a77323204525dc853ad676ed1ffc0b5e0ab2a9 passed the scoped C-consumer provider workflow (run 45762731599/job 139716270205). New head 47930cf80c33682fde3892787a0b37e82317330c adds L69 regressions and exact receipt checks; execution pending. Existing native namespace graph and .oresi admission are metadata/host gates and do not authorize executable namespace imports. No L64–L68 source-consumer certification. NET #280 b1b8d3b46150cdf58bc9a6feb3b88c232b34931b repairs a stale local Pair fixture without relaxing the core identity gate; retry 45854746957 pending. Rust #521 exact-head scoped 31/31 run 45852161756/job 140021039922; not separate-object codegen. All new applicable cells NOT_EXECUTED until actual target receipts arrive.

### Executed receipts and next source-consumer stage (2026-11-26)

- LLVM #951 exact head 47930cf80c33682fde3892787a0b37e82317330c / tree 4052fe6f625a536934159232ce4acc24a9d64e86, run 45854917485/job 140030637110: actual compiler build, both O0/O2, native 42/43/42/43, two unchanged artifacts, seven negative cases per optimization level, source-free relink. C consumer only; L69 scoped pass, not L64.
- NET #280 exact head cc53fa13a941f2e7176ad1dc3e822a0f571fecc8 / tree c6e86d99de03b5d0b863d75001ddb81614bf7721, run 45855106299/job 140031298364: full managed verification and real NativeAOT compiler + linked guest 42, CIL publication marker passed. Existing linked source graph, not interface-only separate object parity.
- LLVM #970 stacked on #951, head 01a3984b2b129c65345225dd2b02f9a58e7380e0 / tree c8bff533eeb5cc6bbb1a0901a07f706fd310d93c: real Atto module-import consumer emitted by compiler, then IR partitioned to retain external physical import slots; independent provider objects renamed to fixture slots, source-free relink, clean differential and missing-provider denial. Initial frontend still needs all source units; graph-local symbol translation is not a production ABI or a verified namespace interface. Exact carrier run 45857384267 pending; no new pass until completed.
- Keep L64 (source-free initial frontend compilation) and general L65–L68 unverified across every applicable target. Do not propagate LLVM host/native receipt or .NET managed/NativeAOT receipt to other backends. HDL static elaboration and realtime bounded deterministic native target must use target-specific applicability and actual artifact evidence; Atto self-host must use actual bootstrap compiler, not host-only models. Related #61 independently tracks adapted fixture receipts, which do not certify canonical namespace linkage.

### Follow-up exact execution and interface/refresh admission (L71–L74)

LLVM #970 exact head 01a3984b2b129c65345225dd2b02f9a58e7380e0 / tree c8bff533eeb5cc6bbb1a0901a07f706fd310d93c (3698 entries), pal-trace-test run 45857384267/job 140039213400: actual compiler and IR auditor built, provider tests O0/O2 passed, real Atto module consumer partition and source-free relink O0/O2 executed 42/43/42/43, consumer and unaffected provider byte-identical, missing provider rejected. Initial source graph and fixture-local symbol translation still required. L64 remains NOT_EXECUTED.

| ID | Required behavior |
|---|---|
| L71 | Immutable verified interface retains exact compiler version, target ABI and policy. A foreign or stale profile cannot authorize typed import binding even when its digest is valid. |
| L72 | A restricted zero-argument signed-i64 public export resolves only through the admitted namespace/module and exact signature. Missing/private export, overload ambiguity, arity/return/effect drift and tampered digest reject; physical names must separate distinct locked package instances. |
| L73 | Editor refresh follows reverse closure of both old and new dependency edges; removal, negative lookup recovery and package-owner rebinding refresh transitive consumers; reordering inventory is a no-op. |
| L74 | Failed inventory admission leaves the last refresh snapshot unchanged; editor refresh is advisory and cannot authorize executable cache reuse. |

LLVM #971 exact candidate 54a9b4bc17aa4b55362b9ae33468ef31543a399c / tree 54a175c2f64e155940ad2f1929779945302ff95a implements restricted host .oresi binding and release-bound symbol derivation, not body-edit-stable logical slots or compiler frontend execution. GCC, Clang and sanitizer replay plus existing guests pending carrier run 45857965303. Rust #647 stacked on approved #521 exact candidate b286e81c23e69499cbd76c29d0ebf8f2df1b5c72 / tree 32970f9b603707c7a963cba58626094f3a32eac5 adds transactional editor refresh planner and actual source-linked tests for missing provider/rollback/rebind/removal. Carrier run 45858095611 pending. No propagation of these scoped results across the other nine targets. All eleven target adapters must record independent actual compilation evidence; HDL static-only and realtime target constraints remain explicit.

### Stable logical linkage and immutable interface admission (L75–L77)

| ID | Required behavior |
|---|---|
| L75 | Independently authorized logical dependency slot and exact release pin are distinct. Compatible body-only replacement preserves the logical link symbol; stale release, distinct side-by-side slots, different link domain and incompatible compiler/ABI/policy reject or remain distinct. Exact artifact admission remains separate. |
| L76 | Canonical interface snapshots are immutable; malformed/sparse/accessor metadata is rejected before source compilation and does not change the canonical digest. For non-JS targets use equivalent malformed input and snapshot ownership checks. |
| L77 | Remove provider implementation source BEFORE initial consumer compilation; compile against the admitted public interface only, bind two compatible releases, execute 42/43 and preserve the old binding or executable and exact consumer bytes. |

LLVM #971 previous exact head 54a9b4bc17aa4b55362b9ae33468ef31543a399c: actual provider and consumer O0/O2 passed; GCC and Clang host interface tests passed, sanitizer link failed because Clang 19 runtime libraries absent (run 45857965303/job 140041213461). Current candidate 71c341ee47ab68199b5892c5e0272d559e8c0317 / tree 9f1b62ddde0f37d1d4507c768af8637be843a017 adds trusted logical-slot metadata and compatible release/side-by-side/stale-pin tests, carrier installs libclang-rt-19-dev, exact-tree run 45860493899 pending. No executable slot or namespace frontend certification.
JS #196 head 00b697c357d7374056aa116bc1adc25c6fd35695 / tree 34939cfe905a2eb788a73d97bd9683ff688aee09 adds immutable dense interface snapshot and real interface-only scalar consumer test with both provider sources removed before compile. Exact full-suite replay 45860654243 pending; no general namespace frontend claim. Rust #647 exact 34/34 passed (17 incremental, 16 multihop, 1 canonical key); promoted ready, parent #521 unchanged. Do not propagate restricted per-target evidence to other targets or conflate metadata with executable linkage. All eleven targets remain in scope.
