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

### Executed interface receipts and stable symbol object linkage (L78–L79)

- LLVM #971 exact head 71c341ee47ab68199b5892c5e0272d559e8c0317 / tree 9f1b62ddde0f37d1d4507c768af8637be843a017, run 45860493899/job 140049931094: full restricted provider and real Atto consumer O0/O2 passed; GCC, Clang and ASan/UBSan host interface and stable-slot assertions passed. L75 host metadata scope only.
- JS #196 exact head 04a82cc1eb5ffad86d3b2f5b6aab31b3238aad41 / tree 20b48afc6ecb58d779fe71cd9adaef0a0a1d5390, run 45860824509/job 140051081731: full 1969/1969, focused 9/9, zero failure/skip. Real restricted scalar consumer compiles with provider source absent and binds compatible versions 42/43/42; L76/L77 restricted scalar API scope, not general namespace imports. Both PRs promoted ready and independent review requested.

| ID | Required behavior |
|---|---|
| L78 | Executable linkage uses actual stable logical slot names rather than graph traversal order; compatible replacement reuses byte-identical consumer and unaffected provider; stale release pin and missing provider deny. A fixture symbol rewrite is an intermediate milestone, not compiler-owned stable slot emission. |
| L79 | Source-owned full suite and backend test ownership remain mandatory; preserve native test in native crate, browser test in browser crate, and no broad policy exemptions. Scoped alternate receipt does not waive executed source failures. |

LLVM #972 candidate b494aa7992af66decb9ce25d76f8e40e9474a64c / tree 28f780b5afbea5119355a3ff228221061ecb0971, run 45861284463: real consumer/provider object relink using verified fixture logical names; initial provider source graph still required, no compiler-generated interface exporter or production lock provenance, pending execution. Rust #521 source-owned jobs 139641203506 and 139641203136 actually failed: nested scanner literals and no-dependency sentence, and misplaced native unary-plus test imported in browser crate. Main-based repair #650 head 8a6d1cda770f93105cfae6d0e6481f5eb8b66aa6 / tree 2e05934444a869bd227fc2f477c36dd052a3e323 preserves native test bytes and exact scanner path exemptions, alternate run 45861475229 pending. No merge bypass. All eleven target adapters remain in scope; none inherits another backend's executable receipt.

Exact LLVM #972 follow-up: run 45861284463/job 140052668021 completed SUCCESS, all eight steps; actual verified fixture-slot native relink O0/O2 42/43/42/43 and unchanged two objects, stale release denied, source graph required; initial interface-only frontend remains not executed. Ready for review. Rust #650 first run 45861475229/job 140053329796: scanner unit tests 6/6 and full repository scan passed, browser full suite exposed another misbound standalone arena-model test (root src/actor_arena.rs exists but is not exported by either crate). Rebound that test directly to the existing host-only source without exposing runtime API; current head 8c50abd4931987e3865aeabc7767c7e583ffa88e / tree 200414404ec31306f0c37dc97ff33d78e5130fa9, run 45861674846 pending. Native unary-plus regression still must execute in native crate; carrier now runs it independently after exact source admission even if browser suite fails. No source check override.

### Compiler-emitted stable local symbols and native/browser repair (L80)
L80 requires real compiler codegen to preserve direct scalar local module identity under import reordering, emit matching provider/consumer physical symbols without object rewriting, reject duplicate module identity/foreign target before IR, and prove native 42/43 selective relink. LLVM #973 candidate 52eb3749e37e0e3cd1d53adbe3bc719ce8a28f4d / tree cb4feda9309a48af9d2f1a8e16c76ce7b4a92c4a adds isolated opt-in hosted loader/CLI profile, direct single public i64() module per unit; provider source graph and IR partition still required; no verified package namespace or interface-only frontend claim. Carrier commit 1b277aa96f824ee268023c64377267ba83be8324 submitted; execution pending.
Rust #650 retry 45861674846 actual policy pass 6/6 + repository scan; browser compile revealed root AoSoA test misbound to old native crate (actual checked function resides in browser crate); native unary plus tests 0/4 because current native parser does not admit prefix plus. Repaired browser test crate reference and native parser iterative identity prefixes preserving checked operand validation, added long prefix regression. Current candidate 40daa3f0f101c09f44b2de3eb7e5c8e6cab36154 / tree 4edc20b6cb6011588fe9c5b91a6652be61f2a967, run 45862705717 pending. Original approved #521 unchanged. All eleven backends retain independent acceptance gates.

### Executed compiler-owned local symbols and canonical inventory admission (L81)
LLVM #973 exact head d306f518ced8c70ebef1045b3eb5d1ed661e7105 / tree c18935facf0577c5c76179559cccb02af1577d95, Mean Muffin run 45863852945/job 140061526815 SUCCESS all nine steps: actual compiler, native provider/consumer tests, GCC/Clang/ASan/UBSan, real O0/O2 compiler-emitted local module names without objcopy, 42/43/42/43 with unchanged two objects, reordered imports stable, duplicate module/foreign target rejected. Prior run 45863061673 failed due GPU-exclusive flag tracker erroneously rejecting all ordinary flags; fixed without weakening foreign target denial. Promoted ready and independent review requested. L80 restricted local source profile PASS; L64 initial interface-only namespace frontend still NOT_EXECUTED for LLVM.
Rust #650 head d36049c59ad208c51fd609d17eb6d09358e54007 / tree ea2621ac10ad17d3bbe8c857e82ac0606238b178, run 45863080894/job 140058860021: native unary plus 5/5 PASS including 1024 prefix admitted and 4096 rejected by existing token budget; policy 6/6 and full scan PASS. Browser full suite still fails declarative_exports due stale native API and CLI references. Neither root browser API nor current native frontend admits the test as written. Preserve test; no suppression or merge override.
L81: normalize source and importer inventories before cache mutation; reject ambiguous canonical importer aliases, null source bytes and inconsistent empty source inventory with import pins. Rejected inputs preserve prior checked and parsed snapshot. Java #1933 candidate 1c3118dbd1db6c24cadc7810cfbb25289ff8f4ce / tree dfd8005ed9b8492e0cdb938b9c52a3c7b2eb9cb9, two added regression cases. Canonical Cloud runs 45864066572 and 45864058843 had steps=null; alternate Mean Muffin run 45864161493/job 140062556826 verified exact tree and executing JUnit/guest tests, pending. All eleven backend adapters remain independently gated; no blanket parity claim.

### Interface-only namespace consumer candidate and canonical inventory receipt (L82)
Java #1933 repaired exact head ac450a8852d74e0379f492497f33534e1cb4c2bf / tree d3b3256f86b0ac38b522c99bffff6c4235e26489, Mean Muffin run 45864514489/job 140063775320 PASS 36/36 no failures/errors/skips; package isolation 7/7, actual namespace guest cold/warm 41,42 and edit/clean 41,43. Earlier 35/36 failure was new fixture using unsupported direct call through explicit named import; fixture now isolates binding rollback without claiming direct-call execution. Promoted ready and independent review requested.
L82: Host-admitted, bounded scalar namespace consumer compilation must resolve actual source calls against immutable verified interfaces with provider implementation sources absent before compilation, using stable logical slot names and emitting declarations only; invalid signatures, arity, missing export and stale release must deny before IR publication. LLVM #976 candidate 8406abb2d138f3d6fa4c1121706d567bbb36de34 / tree 43b1464e905a27e3e6f999aa8228e236859478a3: restricted host-owned one-module-per-namespace scalar expression API, real parser/type/ownership passes with synthetic signature witnesses (no witness body emitted), private codegen declaration-only set, fixed test CLI OFF by default. Test uses fixture .oresi metadata and provider-side objcopy, no consumer partition, no production lock/provenance or arbitrary artifact trust claim. Exact carrier run 45864901355 pending. All eleven targets retain independent gates; L64 not yet verified for LLVM. No private source transfers or source CI overrides.

### Verified interface-only compilation and direct provider slots (L83)
LLVM #976 exact head 8406abb2d138f3d6fa4c1121706d567bbb36de34 / tree 43b1464e905a27e3e6f999aa8228e236859478a3, run 45864901355/job 140065097183 SUCCESS all ten steps. Actual source namespace consumer O0/O2 compiled with provider implementation sources absent, no consumer partition, native 42/43/42, four invalid imports denied. Provider fixture symbol rewriting remains. L82 restricted interface-only compiler API PASS, not general production package authority; promoted ready and independent review requested.
L83: Independently pinned provider source and release must be checked against public i64() interface before emitting the identical physical slot; no provider object rewrite or consumer partition. Compile providers independently, remove provider sources and IR, compile real interface-only consumer, link and execute old/new/old, verify untouched consumer and unaffected provider, reject stale source digest, stale release, wrong signature and four consumer negatives. LLVM #979 candidate ebf468d987a711bb37e0438d16eacb6f38628fa3 / tree 91c29777410412d7736fc774fcabfb9925c0003b, run 45871802516/job 140088815325 exact tree passed, compiler build pending at inspection. Host fixture pins, not registry or object provenance certification.
Current-head drift: Rust #521 reconciled head 65b6fa23fe652938c4e3eafdbcf01bddcf4375a3/tree d08b958cc968a0dd66227014fcd9f284ac4788fd; prior approval belongs to ec166b11e183d1c656b7db5ad8a5cd36503b1a23; fresh independent review requested. Current head includes native declarative-export and unary-plus tests, prior #650 is dirty against newer main. Cranelift #115 reconciled head b028ef3ab8a9cade90867b795bdd102814da5a55/tree 2542a21eeaee76dbc5b2179c9c27de8d0ff46863; both exact trees admitted in Zed run 45871913183, Rust policy step failed and browser step skipped; carrier commit 40f422cee692a801bdd4dca31c5cb8702a65f86f reruns independent full browser suite despite policy failure. No prior receipt or review transferred to new heads. All eleven targets independently gated.
