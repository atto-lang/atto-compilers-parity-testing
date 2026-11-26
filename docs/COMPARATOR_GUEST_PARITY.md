# Comparator exact-source positive guest probe (candidate)
The site comparison PR [#235](https://github.com/atto-lang/atto-lang.github.io/pull/235) pins 33 public .atto fixtures and expected return/stdout in `comparisons/parity-candidates.json`. The current probe checks out that exact site commit/tree, never rewrites fixture bytes, checks all Git blob SHA-1 values and pins each compiler's own main snapshot. It emits per-case observations; only exact positive oracles are matches, not full parity.

`node tools/comparator-guest-probe.mjs <rs|js|llvm|forge|cranelift> <exact-site-checkout> <exact-compiler-checkout>`

Build the compiler in the checked-out repo first. Rust runs native attoc output in the reviewed site's guest runtime; JS executes generated ESM (explicit scalar/pipeline profile); LLVM runs its emitted native executable; Forge uses runtime assembly, not its interpreter-derived constant assembly; Cranelift runs its native JIT and only compares cases with empty expected stdout (CLI return line). Unavailable profiles, rejection, malformed or wrong results remain gaps. This is an exploratory positive probe, not negative coverage or target certification.

Remaining independent execution adapters: Java JVM and Graal Native Image; .NET CoreCLR and NativeAOT; Wasm integration with pinned producer and worker host; Atto Stage-0 and later bootstrap rebuild; HDL simulation/synthesis; realtime admission/WCET. No simulated success, host-only evidence, or zero-step CI counts. Shared negative tests must classify diagnostics and absence of artifacts, runtime traps and cancellation; all target-specific prerequisites need separate proof. Keep exact source/head/tree, toolchain, generated artifact digest, executed runner steps and independent review.

## Second wave

Java JVM and Native Image are separate `java` and `java-aot` lanes (JDK classpath and native compiler host are built first); .NET `net` and `net-aot` compile unchanged source to fresh CIL with explicit trusted stdout permission, then independently build and execute the fixed CoreCLR or NativeAOT host. A failed build, nonempty runtime stderr, or an unrepresentable exit value is not a positive match. Rust now also attempts two exact pinned negative fixtures: immutable reassignment (typed rejection with no Wasm output) and actual checked overflow (runtime panic code 1 and retained stdout). These narrow negatives do not certify other profiles.

## Third wave

Rust completed all 33 positive and both exact negative cases on the reviewed source snapshot in [run 45733832607](https://github.com/declarative-migrations/atto-ci/actions/runs/45733832607/job/139622152383). Java JVM built and attempted all 33; 27 rejected, six executions remained unverified due to host stderr. The new strictly pinned JDK warning classifier records the exact four-line Truffle Unsafe warning as host evidence; every other stderr byte remains a guest-oracle blocker. No JVM security flag is changed. The `js-candidate` lane pins JS PR #161 separately from the baseline and grants bounded stdout only to the function-pipeline source. Re-execute all 33; do not infer a pass from a narrower regression suite.

## Integrated .NET candidate replay

`net-candidate` and `net-aot-candidate` are separate exact-tree lanes for .NET #16 (2ac98a727ee8906fd174514d25cf2b7dc2a9e311). The three source-native suite jobs passed, including exact comparator pipeline return/stdout on both runtimes. Run the complete 33 unchanged cases for updated coverage; do not carry forward baseline counts or infer full parity. The source snapshot and original baseline remain immutable.

## Forge functional C++ candidate

`forge-candidate` pins Forge #45 at f84900637f3d1faaf192688b212a2b7ef52b206e, after normal and sanitizer suites passed. The probe compiles original bytes to C++, builds actual native executables, and distinguishes the emitted pure i64 return wrapper from the explicitly granted stdout/exit wrapper. A wrapper/profile mismatch cannot satisfy either oracle; no guest source is rewritten. The original assembly backend baseline remains separate. Full 33-source replay is required.
