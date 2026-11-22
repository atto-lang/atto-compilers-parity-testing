# AttoLang incremental compilation — 28-system engineering benchmark
**Research checkpoint:** 2026-11-22 · **Status: RESEARCH/ACCEPTANCE PROPOSAL, NOT VERIFIED BACKEND IMPLEMENTATION**.

This is a curated comparison of 28 established or widely used compiler, build, dev-rebuild and bundler ecosystems. **Not a measured popularity ranking, not a controlled speed benchmark, and not 28 equivalent language compilers**. The linked first-party documentation supports the specific mechanism described, and the proposed takeaway is our architectural inference for Atto. Do not assume a bundler's HMR or a build engine's task caching is proof of semantic compiler incrementality.

Cross-reference [machine-readable source matrix](../contracts/incremental-landscape-28-ecosystems-2026.json), [Java source draft #1744](https://github.com/atto-lang/atto-source.java/pull/1744), [Atto spec #213](https://github.com/atto-lang/atto-spec/pull/213), [shared planner #133](https://github.com/atto-lang/atto-common-source/pull/133), and [tracking issue #53](https://github.com/atto-lang/atto-compilers-parity-testing/issues/53).

## Executive assessment

AttoLang is **competitive in its architecture proposals** (package-instance-qualified namespaces, distinct source-file fallback, dependency-aware ABI invalidation, ownership/actor proof boundaries). It is **not yet production-competitive as an implemented incremental compiler**. The Java draft currently provides **in-process** raw/expanded parse caching, selected typed-unit reuse, strict/legacy cache partitioning, dependency and import-binding invalidation, effective exported API fingerprints, atomic publication and ten earlier + five later authored JUnit cases. Source workflow failures with `steps=[]` are **NOT_EXECUTED**, not evidence that these tests pass. The 20 locally tested cases in the shared Python planner prove a reference model, not actual Java, LLVM, .NET or another backend.

The missing center of gravity is a durable incremental query+artifact engine whose cache hits remain sound across process restarts, semantic-profile or package-lock changes, code generation, linker boundaries, actor/ownership effects and all target backends.

### Core architectural differentiation

- **Source change-detection unit:** physical file. A file without an explicit `namespace` is never implicitly grouped with unrelated files.
- **Semantic interface unit:** explicit namespace qualified by the immutable resolved package instance. Multiple verified files may contribute one namespace.
- **Finer dependency unit:** effective exported declaration, public contract, negative name lookup, macro/annotation expansion, type/borrow/effect/actor witness, inline body and specialization reference.
- **Emission unit:** source/namespace/function/IR partition selected by each compiler's proven capabilities; backend output need not share Java's exact artifact granularity.
- **Distribution/link unit:** package/target artifact; a full link may be necessary even when most parsing/typing is reused.
- **Authority unit:** a cached AST is *not* a trusted ownership proof or actor admission; those require versioned validated semantic dependencies.

This combines the *compiler* ideas of Rust/Scala/GHC with the *build system* ideas of Bazel/Buck2/Pants and the fast-feedback needs of TypeScript/Vite/Flutter, without making HMR synonymous with AOT incrementality.

## Comparison: 28 ecosystems

| # | System and upstream documentation | Layer | Observed incremental mechanism | Design requirement for AttoLang |
| ---: | --- | --- | --- | --- |
| 1 | [Rust/rustc](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html) | Compiler | Red/green query DAG, persisted query results and codegen-unit reuse | Query-level invalidation plus persisted IR/object results; avoid marking cached values green without rederived dependencies |
| 2 | [Swift compiler/driver](https://github.com/swiftlang/swift/blob/main/docs/Driver.md) | Compiler | Fine-grained source dependency files and incremental frontend jobs | Record per-source provided/referenced declarations; schedule source jobs independently and revise dependency graph when emitted metadata changes |
| 3 | [Kotlin Gradle plugin](https://kotlinlang.org/docs/gradle-compilation-and-caches.html) | Compiler | Classpath API snapshots and dependent class recompilation | Compare actual exported API separately from implementation text, including non-source dependencies |
| 4 | [Scala/sbt Zinc](https://www.scala-sbt.org/1.x/docs/Understanding-Recompilation.html) | Compiler | Name/member reference hashes and class-based dependency analysis | Refine file/namespace invalidation to exported declarations and symbol-usage edges, preserving inheritance/implicit effects |
| 5 | [TypeScript/tsc](https://www.typescriptlang.org/docs/handbook/project-references) | Compiler | On-disk tsbuildinfo and project-reference declaration boundaries | Persist analysis by package/profile and use verified interface artifacts between build units |
| 6 | [Go cmd/go](https://pkg.go.dev/cmd/go) | Build | Package/action identity build cache and compiled export data | Make package-instance keys stable and reproducible across compiler processes |
| 7 | [Haskell/GHC](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/separate_compilation.html) | Compiler | Interface and per-declaration fingerprints to avoid recompilation | Fingerprint effective exported entities and invalidate only importers of changed APIs |
| 8 | [C#/Roslyn incremental generators](https://github.com/dotnet/roslyn/blob/main/docs/features/incremental-generators.md) | Compiler plugin | Incremental value-provider graph and cache-friendly deterministic transforms | Track annotation/macro-generated declarations as versioned, deterministic query outputs |
| 9 | [Elixir/Mix](https://mix.hexdocs.pm/Mix.Tasks.Compile.Elixir.html) | Compiler | Compile-time vs runtime module dependency tracking | Distinguish semantic/compile-time imports from runtime-only edges; actor protocol consolidation is a distinct output |
| 10 | [Zig compiler](https://ziglang.org/download/0.16.0/release-notes.html) | Compiler | Compiler/watch incremental compilation with linked code emission (some modes evolving) | Keep file/function and target-object invalidation separate; claim stable behavior only after executing supported target modes |
| 11 | [Dart/Flutter dev pipeline](https://docs.flutter.dev/tools/hot-reload) | Dev/HMR | Incremental source reload to running VM during debug | Treat hot loading as separate state-preserving runtime protocol, not proof of native AOT incremental build |
| 12 | [OCaml/Dune](https://dune.readthedocs.io/en/latest/reference/caches.html) | Build | Memoized rule graph and build cache with reproducibility checks | Add randomly sampled cache re-execution and deterministic artifact equivalence checking |
| 13 | [Java/Gradle](https://docs.gradle.org/current/userguide/performance.html) | Compiler + build | ABI-aware compilation avoidance and incremental Java compilation | Reuse unaffected importer outputs when bodies change, but invalidate on effective ABI changes |
| 14 | [C#/.NET MSBuild](https://learn.microsoft.com/en-us/visualstudio/msbuild/incremental-builds) | Build | Declared inputs/outputs and incremental target skipping | Explicit per-step input/output manifests and canonical build action keys |
| 15 | [Java/Maven Compiler Plugin](https://maven.apache.org/plugins/maven-compiler-plugin/compile-mojo.html) | Build | Module/source change detection; coarser incremental modes | Avoid calling coarse module timestamp checks equivalent to semantic compiler incrementality |
| 16 | [Bazel](https://bazel.build/versions/7.3.0/remote/caching) | Build | Content-addressed action cache and optional remote reuse | Hermetic action key includes toolchain, options, inputs, environment and declared dependencies |
| 17 | [Buck2/DICE](https://buck2.build/docs/insights_and_knowledge/modern_dice/) | Build | Dynamic memoized dependency computation graph and incremental actions | Use dynamic dependency edges, incremental graph recomputation and safe parallel scheduling |
| 18 | [Pants](https://www.pantsbuild.org/stable/docs/introduction/how-does-pants-work) | Build | Fine-grained file dependency inference, memoized hermetic engine | Package file-level source membership plus verified dependency inference and selective invalidation |
| 19 | [Ninja](https://ninja-build.org/manual.html) | Build | Explicit file graph, depfiles and restat-based incremental actions | Emit real depfiles/output manifests; avoid changed timestamps triggering unchanged downstream outputs |
| 20 | [CMake/Ninja generator](https://cmake.org/cmake/help/latest/command/add_custom_command.html) | Build | Generated Ninja target graph with depfile integration | Support standardized build graph integration without inventing a new unsupported build orchestrator |
| 21 | [webpack 5](https://webpack.js.org/configuration/cache/) | Bundler | Persistent filesystem module graph cache and dependency-based invalidation | Persist module/namespace transforms and invalidate when config, plugin or file dependency changes |
| 22 | [Vite](https://vite.dev/guide/api-environment-instances) | Dev/HMR | Environment-specific module graphs and precise HMR invalidation | Keep isolated target/runtime-profile graphs; version invalidation across client/worker/host actors |
| 23 | [esbuild](https://esbuild.github.io/api/) | Bundler | Persistent context rebuild/watch and incremental in-process graph reuse | Fast in-session reuse is valuable but not a substitute for process-restart persistence |
| 24 | [Rollup](https://rollupjs.org/configuration-options/) | Bundler | Watch rebuilds and reusable module cache | Model target fragments, dynamic imports and accurate external dependency invalidation |
| 25 | [Parcel](https://parceljs.org/features/development/) | Bundler | Disk cache of transforms and granular invalidation of config/plugin inputs | Treat codegen plugins/macros/feature configuration as first-class cache inputs |
| 26 | [Angular compiler/build](https://angular.dev/tools/cli/build-system-migration) | Compiler + build | Incremental AOT-oriented build system and partial library output | Separate ahead-of-time analysis/emission/linking and preserve correct generated dependency provenance |
| 27 | [Nx](https://nx.dev/docs/getting-started/how-nx-works) | Build | Task/affected project graph and hash-based local/remote result replay | Explain exact rebuild reasons and permit optional trusted workspace/CI artifact reuse |
| 28 | [Turborepo](https://turborepo.dev/docs/crafting-your-repository/caching) | Build | Global plus per-task hashes with local and remote task caches | Include environment/toolchain dependencies in cache keys and ensure reproducible output manifests |

## Hard priorities and acceptance criteria

| Priority | Work | Scope | Completion gate |
| --- | --- | --- | --- |
| P0 | **Prove one correct Java frontend slice** | verified 15-case JUnit, real source/diagnostics, warm/no-op/negative import/borrow effects, no bypass of actor admission | actual nonzero runner steps + clean-vs-incremental diagnostics parity + independently reviewed CI path |
| P1 | **Bind packages/namespaces and precise semantic dependencies** | manifest+lock package-instance ID, one unit per unnamespaced file, shared namespace interface graph, negative lookups, nominal actor/ownership/effect/inline edges, SCC interface-first | 12-file/3-namespace plus unnamespaced fixture with exact rebuild-set oracle and negative cases |
| P2 | **Persist deterministic semantic/compiler graph** | versioned CAS for parse/typed/HIR/MIR/declaration fingerprint, restart/recovery, corruption and toolchain-profile detection, atomic publish and concurrent cancellation | restart in a second process reports true disk hits, corruption rechecks, no stale ownership evidence |
| P3 | **Incremental codegen, linking and target artifacts** | per backend IR/object/ESM/Wasm/CIL/RTL artifact hashes, inline specialization graph, link vs compile counters, AOT profile partitioning | real emitted artifact + guest output/diagnostic parity and targeted recompilation for supported modes |
| P4 | **Build UX, hermeticity and optional trusted remote cache** | --explain-incremental stable machine report, local cache first, target/toolchain/env/key provenance, optional verified remote cache, p50/p95 timings and RSS | reproducible benchmark receipts, process-restart reuse and hermetic cache adversarial tests across supported backends |

### Reference dependency graph

Each compiler backend should consume a common **versioned dependency graph** rather than independently inventing import semantics:

```text
Source snapshot (package manifest + lock, full file membership, profiles)
  -> source file hashes / import resolutions / negative lookups
  -> raw + expanded AST nodes
  -> nominal symbol/interface/namespace queries
  -> types + effects + borrow/actor-state admission queries
  -> HIR/MIR per proven compilation unit
  -> target-specific codegen fragments / object modules
  -> target link / executable / dynamic hot-load artifact

Dependency edges:
  source-content | symbol-reference | import-binding | negative-lookup
  namespace-membership | interface-ABI | inline-body | specialization
  macro-generated | ownership/effect/actor-witness | toolchain/profile
  codegen-target | link-input
```

The graph must distinguish **red input that leaves the derived value unchanged** from a genuinely changed derived value. It may reuse an output only if *all relevant versioned inputs and semantic prerequisites* remain valid. If the compiler cannot prove an edge complete, conservatively recheck or report unsupported; do not convert weak cache evidence into a success receipt.

### Proposed deterministic cache identity

Include ``schema-version`, `compiler-build-SHA`, `backend-identity`, `backend-target-triple`, `runtime-and-ABI-revision`, `semantic-parser-profile`, `ownership/effect/actor-permission-flags`, `package-instance-id-and-lock-snapshot`, `namespace-or-source-unit-id`, `source-digest`, `resolved-import-and-negative-lookup-digest`, `exported-ABI-and-inline-body-digests`, `macro/plugin/generated-input-digests`, `optimization-flags`, `toolchain/SDK-version`, `deterministic-declared-environment-digest``. Toolchain and semantic schema changes always invalidate incompatible artifacts. Document exactly which environment variables influence deterministic output and reject undeclared nonhermetic reads. Persistent entries require immutable content IDs, checksums, bounded serialization, atomic manifest promotion and a recoverable last-known-good snapshot. Remote caching is an **optional later extension**, never a substitute for local correctness.

### Adversarial build scenarios and exact expected results

| Case | Mutation | Required observable behavior |
| --- | --- | --- |
| A1 Cold vs incremental same inputs | Fresh build and same-process/restarted incremental must match executable output, diagnostics and errors | Fresh build and same-process/restarted incremental must match executable output, diagnostics and errors |
| A2 No-op same process | Zero reparses/rechecks/recodegen when all verified inputs unchanged | Zero reparses/rechecks/recodegen when all verified inputs unchanged |
| A3 No-op fresh process | Real on-disk cache hits and zero unnecessary compiler actions | Real on-disk cache hits and zero unnecessary compiler actions |
| A4 Single no-namespace file body edit | Only its file semantic/codegen unit changes | Only its file semantic/codegen unit changes; unrelated no-namespace files unaffected |
| A5 Two independent files no namespace | Never collapse into a phantom global namespace | Never collapse into a phantom global namespace |
| A6 Single file in a multi-file namespace | Reparse only edit, update shared namespace effective interface and invalidate only proven dependents | Reparse only edit, update shared namespace effective interface and invalidate only proven dependents |
| A7 Reorder public declarations | No importer semantic invalidation if public effective ABI unchanged | No importer semantic invalidation if public effective ABI unchanged |
| A8 Public named parameter/effects/borrow signature edit | Invalidate all bound consumers and inherited semantic obligations | Invalidate all bound consumers and inherited semantic obligations |
| A9 Actor state/concurrency protocol edit | Invalidate actor witnesses, linked imports, supervisor protocol and codegen when relevant | Invalidate actor witnesses, linked imports, supervisor protocol and codegen when relevant |
| A10 Same target set, imported symbol rename | Invalidate importer when source import declaration identity changes | Invalidate importer when source import declaration identity changes |
| A11 Negative lookup becomes valid | Invalidate cached unresolved-name assumptions after addition or remap | Invalidate cached unresolved-name assumptions after addition or remap |
| A12 Transitive inline/specialization edit | Recompile inline consumers even when outward API unchanged | Recompile inline consumers even when outward API unchanged |
| A13 Macro/annotation generated declaration edit | Invalidate generated ABI and consumers, including hidden files, config, plugin, environment deps | Invalidate generated ABI and consumers, including hidden files, config, plugin, environment deps |
| A14 Add/delete/move source file | Evict stale members/output and rederive namespace/package membership | Evict stale members/output and rederive namespace/package membership |
| A15 Add/remove explicit namespace header | Move member deterministically | Move member deterministically; no old namespace ghost |
| A16 Switch locked package version | Resolve fresh package instance identity | Resolve fresh package instance identity; prevent cross-version cache reuse |
| A17 SCC/cyclic namespace imports | Deterministic interface-first handling or explicit unsupported diagnostic | Deterministic interface-first handling or explicit unsupported diagnostic |
| A18 Change target architecture/ABI/SDK/optimization | Never reuse incompatible native, .NET, JVM, Wasm or RTL artifacts | Never reuse incompatible native, .NET, JVM, Wasm or RTL artifacts |
| A19 Toggle strict/legacy and permissions | Never reuse parsed/typed borrow authority across mismatched profile or permission settings | Never reuse parsed/typed borrow authority across mismatched profile or permission settings |
| A20 Failed parse/type/ownership check | Do not publish partially valid cache state | Do not publish partially valid cache state; prior working cache recoverable |
| A21 Corrupt/truncated/artifact checksum mismatch | Reject/evict bad entry and fully regenerate deterministically | Reject/evict bad entry and fully regenerate deterministically |
| A22 Concurrent edits while build in flight | Pin immutable snapshot | Pin immutable snapshot; cancel, discard stale result, and preserve atomic publication |
| A23 Clean/incremental Java JVM actual guest | Assert same stdout, stderr, exit status, diagnostics and required object hashes | Assert same stdout, stderr, exit status, diagnostics and required object hashes |
| A24 Clean/incremental Native Image actual guest | Check AOT runtime differences, retained object reuse when truly supported, and final binary behavior | Check AOT runtime differences, retained object reuse when truly supported, and final binary behavior |
| A25 Cross-backend codegen parity | Run equivalent supported fixtures for Java, LLVM, .NET, Forge, Rust/JS/Wasm profiles | Run equivalent supported fixtures for Java, LLVM, .NET, Forge, Rust/JS/Wasm profiles; unsupported gated explicitly |
| A26 Hermetic optional remote cache | Reject mismatched toolchain, unauthorized artifact or unsound remote cache proof | Reject mismatched toolchain, unauthorized artifact or unsound remote cache proof; no token exposure |
| A27 Reproducibility sample | Randomly force clean recomputation of nominal cache hit and compare resulting fingerprint/output | Randomly force clean recomputation of nominal cache hit and compare resulting fingerprint/output |
| A28 Profile explainability | Emit machine-readable exact causes of reparses, rechecks, codegen, link and cache reuse | Emit machine-readable exact causes of reparses, rechecks, codegen, link and cache reuse |

### First practical 12-file fixture

Start with **12 Atto source files**: 10 assigned to 3 explicit package-qualified namespaces and **2 with no namespace**. Verify all are compiled and executed on a clean baseline, then make a body-only change to one of the two unnamespaced files:

- Expected parse/semantic rebuild-set: **only that source file**, subject to independently proven macro/inline or whole-program dependencies.
- Expected reused parse/type units: the **other 11 files**, if their effective declared dependencies are unchanged.
- Expected codegen/link: report separately; the edited file's fragment and any required target relink may be regenerated without counting the whole program as recompiled.
- Expected behavior: same output/diagnostics as a new clean build of **the edited snapshot**. Do not compare edited output to the old pre-edit artifact.

Then repeat with an interface change, inlined body change, namespace move, negative lookup, effect/ownership/actor change and package lock revision. A compiler whose graph ignores these dimensions is *not* correctly incremental, even if a hot no-op build is fast.

### Backend-specific proof boundary

| Backend | Minimum realistic acceptance |
| --- | --- |
| Java/GraalVM | Exact-head JDK25 JUnit stepful run, JVM clean-vs-incremental .atto guest, strict-vs-legacy isolation, Native Image guest separately proved |
| LLVM | Real .atto -> LLVM object(s) -> native binary; hash which object(s) were reused; ASan/UBSan and target profile separation |
| .NET | Actual generated CIL + Linux/Windows Native AOT guest comparison; target/profile/SDK hashes; no false shared Native Image cache |
| Forge | JIT/AOT cache key separation, actor/borrow gate and compiled guest parity |
| Rust-authored source compiler | Actual Rust compiler frontend/IR and host/Wasm output; avoid counting compilation of the Rust compiler itself |
| JavaScript/Wasm | ESM/Wasm fragments, worker actor protocol versioning, true package/target remap and cold-vs-warm guest checks |
| Cranelift/HDL/bootstrap/realtime | Supported subset + explicit `UNSUPPORTED` for missing functionality; no fabricated green complete matrix |

### Performance measurement (after correctness)

Capture source/namespace sizes, compiler/toolchain revision, fixed CPU/RAM, profile, warm state and source tree SHA; run cold/no-op/single-body/API/inlining/namespace/cache-corruption cases repeatedly and report medians and p95, CPU and peak RSS. Also report `reparsed`, `rechecked`, `re-IR`, `re-object`, `relink`, phase-specific cache hits and **the reason for every invalidation**. No speculative 5x, 10x or “millisecond” marketing targets until comparable measured data exists.

**Safety and CI:** source `main` and cross-org private source exporting have a separate [security hold #65](https://github.com/atto-lang/atto-source.java/issues/65). This research work does **not** authorize a private source transfer, alter security policy, mark any compiler as fully supported or claim passing stepful native tests.

## Recommended order of implementation

1. Prove current Java in-memory cache and fix any test failures; reconcile overlapping Java #697 before merging.
2. Integrate manifest package ownership and multi-file namespaces from Java #1536/#1545 and the shared planner #133; emit precise dependency edges.
3. Add a versioned local disk query/CAS with process restart, corruption and schema tests; prohibit persisted unverified semantic authority.
4. Introduce backend-specific artifact/codegen reuse one target at a time, with honest full-link reporting.
5. Add machine-readable `--explain-incremental`, fixed fixture timing, clean-vs-incremental guest oracles, and optional isolated trusted remote caching.

**Bottom line:** AttoLang can exceed many conventional build systems in *semantic precision* if its actor/ownership/effect dependencies are first-class. Today, that is a design direction and partial Java frontend slice, not an executed parity outcome.
