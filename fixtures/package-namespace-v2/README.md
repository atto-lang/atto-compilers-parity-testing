# Shared pinned multi-file namespace fixture

**Source-only conformance input; no executed backend success is claimed.**

Ten provider files explicitly listed by `checkout/.ores.pkg.toml` define
three distinct namespaces: billing (4), identity (3), telemetry (3).
The separate client package imports one pinned billing namespace; its expected
standard output is the four lines `0`, `1`, `2`, `3` in order.

Dependency manifest SHA-256: `5986c589ab92259065f278ae6771e86065aa4f10776bfd1e55dbe31e7003d394`
Sorted path/newline/source-SHA256/newline digest: `e6e794b0a96d94797f571a8b29ae69f9675bacb6818d9d9862c4213bf79f9d50`.
Any source or manifest mutation requires a corresponding updated digest.

**Positive cases:** ten provider files across three namespaces; four exact
billing providers; valid same-package `from "+/"` resolution; exact pinned
cross-package resolution; dependency-first, once-only initialization of
reachable units only; ABI-aware cache invalidation.

**Negative cases:** unlisted source injection, foreign package symbol,
unauthorized dependency alias, wrong/mismatched pins, duplicate provider,
wrong type or arity, qualified `payments.init()` invocation, and stale ABI.
Unreachable modules with `init` must not execute.

Java/GraalVM #2263: expected positive, requires executed test.
LLVM #1179 and .NET #248: fail closed now; functional positive parity pending.
JS #108, Wasm #183, Forge #141: explicit fail-closed namespace admission.
A safe rejection is not a functional parity pass.

Every result must bind the exact compiler and fixture Git trees, job/run
attempt, runner, executed steps, negative diagnostic class, and guest output.
Zero-step failures never qualify compiler correctness. This synthetic fixture
does not authorize cross-organization copying of private implementation source.
