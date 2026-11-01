# Five-backend Oreslang scalar intersection and native stdlib harness roadmap

Status: **authored candidates, not verified guest passes** (2026-11-01).

This repo now inventories six backends: Java/GraalVM, LLVM, Cranelift, core Wasm, .NET Native AOT, plus the existing JS transpiler. The user's immediate parity priority is the first five. This is an explicit distinction between **compiler capability** and **actual exact-head execution evidence**.

## Six new source-language regression oracles

| Case | Native Oreslang scenario | Expected signed integer |
|---|---|---:|
| 32 | grouped addition/multiplication and final addition | 42 |
| 33 | subtraction versus multiplication precedence | 42 |
| 34 | unary plus and digit separators | 42 |
| 35 | signed i64 MAX minus another in-range positive literal | 42 |
| 36 | signed i64 MIN + MAX + 43, preserving eager left-to-right checks | 42 |
| 37 | `Integer.bitXor(40, 2)` as a named builtin, not a caret operator | 42 |

These are six independent **`.ores` programs**, not host-language assertions. All six are `candidate` cases until the unchanged bytes have compiled and executed under an independently reviewed backend and exact source/compiler tree. An arithmetic oracle does **not** prove a backend supports it.

## Capability boundaries and expected gaps

| Backend | Smallest plausible native guest acceptance | Entire 30-test stdlib suite |
|---|---|---|
| Java/GraalVM | scalar main return, plus canonical testing.ores branch still requires reconciliation | Unverified until exact-head compiled guest receipts and independent review |
| LLVM | native `.ores` scalar compiler, existing compile-aot adapter | Requires typed stdlib collections/closures/modules and runner support; no claimed pass |
| Cranelift | one `main()` scalar i64 JIT, no source import graph | **Unsupported today**: TestSuite, imports, classes, collections, actors, async |
| Wasm | LLVM-compiled pure scalar `main() -> i64` via pinned Wasm bridge | **Unsupported today**: zero-import core Wasm cannot run current stdio/test-class harness |
| .NET Native AOT | restricted source scalar assembly linked/published to native host | Full stdlib `TestSuite` and guest execution not established by CIL emission |

A backend with a deliberate unsupported-feature diagnostic is *safer* than silently running a host oracle, but it remains a parity gap. Do **not** substitute Rust/C#, Java, Node, Python, mocked receipts, a stub compiler, or a zero-step GitHub Actions job for executing Oreslang guest bodies.

## Implemented source-local draft gates (2026-11-01)

All five targets contain the **same six Git blobs** and the same three-column `oracles.tsv` manifest (manifest Git blob `6b9d346af9b5495f1bfd1ed93b92db2b77526af5`). These are source identities only.

| Target | Draft PR | Source-local compiler-backed requirement | Current observed guest pass |
|---|---|---|---|
| Java/GraalVM | [source.java #2437](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2437) | Actual Maven Java compiler/runtime JAR + native carrier, `OresMain --mode=aot --check` then guest exit 42 | Not verified |
| LLVM | [source.llvm #1288](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1288) | Exact-head LLVMc -> LLVM IR -> Clang native executable, guest exit 42 | Not verified |
| Cranelift | [source.cranelift #28](https://github.com/ores-truffle-oreslang/oreslang-source.cranelift/pull/28) | Actual Cranelift IR and native JIT, exact `42\n` stdout, compiler SHA-256 | Not verified |
| WebAssembly | [source.wasm #257](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/pull/257) | Manually authorized pinned real LLVM -> core Wasm, Wasm artifact SHA-256, zero-import i64 guest result 42 | Not verified |
| .NET Native AOT | [source.net #321](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/321) | Manual six-case Native AOT compiler + CIL -> native host, executed `42\n` stdout, release-native admission | Not verified |

The central six-source matrix and result-oracle adapter are in [parity #20](https://github.com/ores-truffle-oreslang/oreslang-compilers-parity-testing/pull/20). No private trees were mirrored outside their source organization. Different backends have **different process oracles**: Java/LLVM return 42 as process exit status, while Cranelift/.NET print 42 with exit 0; Wasm returns a typed i64 directly. Comparing these without respecting their ABI would produce false failures or false parity.

### Current CI admission evidence

- Central parity run `41333862139` and Cranelift run `41333866546`: rerun attempts 2 completed as failures with **zero recorded job steps**.
- Inspected exact-head representative jobs on Java `41356577760`, LLVM `41355944629`, Cranelift `41355663325`, Wasm `41356211238`, and .NET `41356405723` also have **zero executed steps**.
- Therefore **0 of 30 backend/case cells** have verified guest-execution evidence in this round. This does *not* imply a semantic compiler fail. All source-local PRs remain drafts; do not merge based on source-inventory tests.
- The .NET six-case full Native AOT lane and the Wasm real-LLVM lane are deliberately manual/pinned to avoid spending unapproved minutes or exposing cross-private-source credentials to PR workflows. They require a trusted explicitly dispatched run once runner admission is repaired.

## Portable execution plan

1. **Exact source bytes:** derive the case SHA-256 from the repository Git tree, capture the compiler executable SHA-256 and verified backend source tree before execution. No private cross-organization mirroring without authorization.
2. **Compile:** Java invokes its reviewed frontend; LLVM invokes `scripts/compile-aot.sh`; Cranelift invokes `orescranelift ir`; Wasm invokes the pinned LLVM-to-Wasm compile bridge; .NET compiles source to CIL. Record actual executed compiler steps and diagnostics.
3. **Run:** Java executes the guest; LLVM runs the native artifact; Cranelift runs its **separate** native-JIT compile-and-run command; Wasm runs the SHA-256-pinned WebAssembly output in its zero-import worker; .NET must link/publish a genuine Native AOT executable and run it. Compile-only, mocked, host-model or generic failure are not passes.
4. **Classify:** candidate cases matching an oracle remain exploratory until independently reviewed. A positive guest result must satisfy both the backend process semantics and its oracle. Cranelift prints `42\n` with exit 0; existing other backend adapters compare the guest's process-exit result. The runner does not silently reinterpret the other backends as Cranelift.
5. **Report:** keep per-case exact source/compiler/backend IDs, job run attempt, nonzero executed steps, exit status and stdout/stderr, native artifact digest, and reviewer. Missing source or step evidence = `unverified`.

Run a six-source **inventory** with (not guest execution):

```sh
cargo run --locked --manifest-path tools/parity-runner/Cargo.toml -- \
  --backend cranelift --case-id 32 --case-id 33 --case-id 34 \
  --case-id 35 --case-id 36 --case-id 37 --output cranelift-inventory.json
```

For actual candidate execution, see `docs/BACKEND_ADAPTERS.md`. An executable adapter and the explicit `--execute --allow-unvalidated` flags are required; output is marked exploratory, not independently validated parity. The full six-backend 55-case inventory contains 330 case/backend cells; it is **not** 330 passing tests.

## How this relates to the 30 native stdlib tests already authored

Existing draft consumer suites: `oreslang-data-science` PR #22, `postgres-orm.ores` #16, `oreslang-sg-stategraph.ores` #8, `oreslang-logic.ores` #6, and `oreslang-forward-proxy-demo.ores` #33. Their 30 tests use `TestCase`, `TestSuite`, `run_suite`, native typed assertions, a summary, and `require_success` from `stdlib/testing.ores` or its vendored source. The central canonical Java work #2393/#2395 still requires reviewed reconciliation.

**Do not copy these private source-bearing PR branches into an unrelated test org** to get Actions capacity. Each repo must first validate its own exact-head compiler guest execution, and only later should the same reviewed source/import closure be executed by each supported backend with proof of per-backend compiler identity.

The next semantic porting work is: stable core `TestOutcome` and typed assertion return values; static/typed list and TestSuite class semantics; fail-closed `check_int_cases` for empty cases; `require_success` for all-skipped suites; bounded machine output; actor TestKit/async fixtures only once structured concurrency and timer semantics exist. **No v2 actor/async TestKit implementation is asserted here.**
