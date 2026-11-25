# Pinned Java reference execution

The Rust `java-reference` adapter prepares and launches a clean, commit-pinned
Java reference compiler. It never copies, flattens or rewrites guest source.
The current corpus contains 53 cases. The reference adapter can attempt all of them
with `--allow-unvalidated`; matching results remain explicitly exploratory.
No validation status is promoted automatically, and exploratory-only execution
cannot exit successfully as a parity gate.

## Reproduce on macOS or Linux

Use a disposable clean compiler checkout: preparation runs `mvn clean package`
with `-DskipTests`, then resolves the runtime classpath. This is a compiler build,
not proof that the compiler's own test suite passed. Supply JDK 25 and Maven.

```sh
git clone https://github.com/ores-truffle-oreslang/oreslang-source.java.git /abs/java-reference
git -C /abs/java-reference switch --detach 3210aeff07597035a4ed815ebbdf39720a3266c1
cargo build --release --locked --manifest-path tools/parity-runner/Cargo.toml --bins
tools/parity-runner/target/release/java-reference prepare \
  /abs/java-reference 3210aeff07597035a4ed815ebbdf39720a3266c1 \
  /abs/jdk-25 /abs/java-bundle.json
tools/parity-runner/target/release/java-reference execute \
  /abs/java-bundle.json /abs/java-results.json
```

`execute` selects only Java, uses a 30-second timeout per compiler/runtime
invocation, and returns the runner's exit code. An exit of 1 with a published
report means corpus expectations were not all met; inspect every record.
Other backends are excluded from this reference-only run. The default corpus
audit still enumerates all five backends (265 inventory records).

Preparation records the compiler commit/tree, JDK/Maven versions, and SHA-256
pins for the compiler JAR, every runtime-classpath JAR, native carrier library,
Java executable, JDK module image, VM library and release metadata. Execution
checks those pins and explicitly selects the native carrier library.

The launcher calls `OresMain --check` and `OresMain --mode=aot` directly through
Java. This mode disables guest JIT; the host is still a JVM, not Native Image.
Maven is not in the per-case invocation path. Integer guest results retain the
CLI exit-status oracle; void-main checks retain their stdout-prefix oracle.
Adapter errors use reserved exit status 125 and are infrastructure failures.

The report's sibling `.identity.json` binds its SHA-256 to the compiler/runtime
pins, adapter/runner binary hashes, and corpus commit/tree plus tracked diff
identity. Keep both files. Bundles contain absolute machine-local paths and are
not portable installations. Use a trusted bundle and a stable checkout during a run.

The runner drains stdout/stderr concurrently and retains timeout diagnostics.
On Unix it terminates the entire adapter process group on timeout, including Java.
Missing adapters, skipped validation, signals, pin mismatches, crashes and timeouts
cannot count as successful required execution.

## Repository refresh, 2026-10-28

All five local compiler repositories and the corpus repository were fetched from
their GitHub `main` refs before the final reference run. Existing local edits and
checkouts were preserved. The Java build uses a separate fresh clone.

| Repository | Fetched main commit |
|---|---|
| Java | `3210aeff07597035a4ed815ebbdf39720a3266c1` |
| LLVM | `06d73764d01a48b25857980a878088612b19e1c4` |
| WASM | `76426512305e07464032710034bcf1a0a9449166` |
| .NET | `37690786a5ae2e53e7f98e5255d928906817f0fc` |
| JavaScript | `354c0cbdded54aaa6b9995d3137ca7f2a0b21407` |
| Corpus base | `cf999b6134164afdd76d9e08b401c6b7b53b1730` |

This refresh does not constitute execution evidence for the four other backends.

## Historical 49-case reference baseline (before cases 32–35)

An earlier pinned compiler run of the 49-case corpus produced 23 exploratory matches, eight classified exploratory
negative matches, and 18 compile failures. That historical runner exited 1. None of those
results was promoted to a parity pass. All `.ores` source bytes remained unchanged.
This baseline is **not** evidence for checked-i64 cases 32–35.

| Compile gap | Cases |
|---|---|
| Named-call argument syntax rejected | `06`, `named-parameters`, `optional-parameters` |
| Legacy mutable-borrow spelling rejected | `20`, `23`, `24` |
| Actor declaration syntax rejected | `30`, `actor-system` |
| Static select `over ... end` rejected | `31` |
| Numeric operands rejected in inferred helpers/lambdas | `import-cycles`, `order-of-operations`, `fuzz-seeds`, `mutex-locking`, `anonymous-functions` |
| Await operand not accepted as future/actor reference | `task-blocks`, `nb-match`, `nb-switch` |
| Imported trait not found among exported declarations | `classes-traits-casting` |

The eight negative matches are legacy cases `13`–`17`, `29`, and the two `nb`
syntax probes. Diagnostic needles are grounded in this compiler's source and
observed diagnostics. Ordinary nonzero exits do not satisfy negative expectations.

Next, reconcile each gap with the canonical language contract and compiler tests.
Do not edit a test merely to make one backend green. Independently review the
23 runtime oracles and eight negative expectations before changing validation status.
