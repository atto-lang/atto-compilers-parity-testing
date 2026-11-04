# Execution and parity gate

The checked-in corpus is fail-closed. The default CI audits 55 legacy programs (including main's four checked-i64 boundary cases), 16 positive multi-file checks and 2 negative probes, yielding **73 logical cases and 438 mandatory backend/case inventory pairs** across Java, LLVM, WASM, .NET, Forge and Cranelift. The 73 optional JavaScript entries are audited separately. It makes no compiler-parity claim unless a case is explicitly validated and an exact backend adapter executes it.

## Rust-only tooling

Oreslang organization policy prohibits Python tooling. The parity harness is implemented in Rust under `tools/parity-runner`. Do not introduce Python, Python actions, or Python helper scripts.

## Exploratory execution

Each backend uses two command templates:

- `ORES_PARITY_<BACKEND>_COMPILE`
- `ORES_PARITY_<BACKEND>_RUN`

Tokens: `{source}`, `{artifact}`, `{workdir}`. Templates are split into argv without a shell. Supported backend names are LLVM, JAVA, WASM, DOTNET, FORGE, CRANELIFT and optional JS. The default inventory selects the six required compilers only.

Example:

```sh
export ORES_PARITY_JS_COMPILE='node /absolute/oreslang-source.js/bin/oreslang-jsc.mjs {source} --out={artifact}.mjs'
export ORES_PARITY_JS_RUN='node /absolute/run-esm-main.mjs {artifact}.mjs'
cargo run --manifest-path tools/parity-runner/Cargo.toml -- --backend js --execute --allow-unvalidated --output js-exploratory.json
```

Exploratory matching is **not** a parity pass. All executed unknown/unsupported/exploratory status values must exit nonzero; only independently validated `pass` or `expected_reject` results satisfy the semantic gate. A no-`--execute` inventory never constitutes guest execution.

## Promotion gate

A case becomes `validated` only after canonical syntax is confirmed against the reference frontend, its semantic oracle is independently reviewed, and the named feature is genuinely exercised. Negative tests require a diagnostic category, not merely nonzero exit status. Unsupported features, skips, timeouts, missing runners and infrastructure failures never count as passes.

For funded-runner fallback, mirror only an identical authorized Git tree into an approved test organization, verify tree/blob identity, prohibit secrets in untrusted workflows, and record exact compiler/runtime SHAs. Mirroring a private tree discloses it to the destination organization, so authorization is required.

## Bounded compiler and guest output

Each compiler and guest process has independent **1 MiB stdout and 1 MiB
stderr** capture budgets. The parity runner continues draining both pipes after
the limit to avoid deadlocking a child that fills OS pipe buffers, but stores
only bounded prefixes. If either stream crosses its budget, the invocation
records `output_limit_exceeded` with no usable exit status. The case is a
`compile_infrastructure_failure` or `runtime_infrastructure_failure`,
**never** `pass`, `expected_reject` or a valid diagnostic classification.
Timeouts and missing runners remain non-passes.

This guards the evidence harness against uncontrolled compiler diagnostics or
guest output; it is not evidence that any backend compiled a particular
`.ores` program. The unit tests include exact-boundary, oversized stdout,
oversized stderr, and timeout cases. Only actual runner steps on an exact
source head constitute executed test evidence.

## Known debt

The Java reference adapter can now execute the full corpus exploratorily; see
`JAVA_REFERENCE.md`. No cases are promoted by adapter execution alone. The first
pinned reference run exposed corpus/compiler syntax and type-checking gaps;
these must be resolved against canonical contracts without backend-specific rewrites.

Case 31 is the Java-reference `do select ... over ... end` target. Explicit fail-closed rejection on LLVM/Wasm/.NET/Cranelift/JS is useful capability evidence, but remains a parity gap rather than a pass until those backends execute the same source semantics.

## Intentional capability rejects

When an execution case is expected to succeed eventually but a backend still
fails at a reviewed select capability boundary, the runner records
`intentional_capability_reject` instead of the less-informative
`compile_failure`. The diagnostic still must classify as
`unsupported_select`.

This status is **not a parity pass**. It remains a hard failure for an executed
required-parity run; it exists only to distinguish a deliberate fail-closed
backend boundary from malformed parsing, crashes, wrong diagnostics, or
semantic execution failures.


## Multi-file corpus

The Rust runner requires exactly three distinct direct local imports in each positive
check, fingerprints the full reachable local import graph (including cycles), and
pins the `PARITY_OK|<check-name>|` stdout prefix. This adds 16 whole-program
probes and two explicit compile-reject probes for `nb match` / `nb switch`.
Those two probes now carry the `unsupported_nb_prefix` diagnostic category and
currently remain `unvalidated`. A compiler returning a generic
nonzero exit does not constitute parity.

Case 31 (static select delimiter equivalence) remains a Java-reference target,
not six-backend execution proof. The `intentional_capability_reject` status
is a fail-closed gap, never a passing record.

## GLM four-backend execution gate

The GLM cases `32`–`51` are required on all four Oreslang targets, but
registration in the shared corpus gives **zero validated compiler outcomes**.
For each of Java/Graal, LLVM, Wasm, .NET and Cranelift:

1. Pin original source Git tree plus the exact compiler/build/runtime SHAs.
   Verify the compiler/runner provenance against an independently approved
   manifest. Do not export private compiler source across organizations.
2. Compile the same 20 immutable `.ores` source files. For LLVM, execute
   each supported case under O0 and O2 with an actual native executable.
3. For Wasm, pin LLVM cross-compiler and emitted Wasm SHA, then run on a
   Wasmtime-backed approved isolated core environment; unavailable binary32
   support is an explicit *gap*.
4. For .NET, require OresIR, emitted CIL, linked NativeAOT host and real
   execution. A CIL-only interpreter check is not a NativeAOT pass.
5. Capture the guest's actual returned 42, unexpected errors, classified
   unsupported diagnostics and all nonzero results. Fail closed if a backend
   is absent, times out, or the GitHub job has zero executed steps.

The first required matrix has **20 × 5 = 100 backend/case outcomes**;
legacy JS testing stays a fifth, independent target. Nothing in this PR
establishes the required 80 executions. Source-only Rust unit tests are
not cross-backend parity evidence.

## Reconciled six-compiler coverage

The signed-i64 programs from `main` retain IDs 32–35. Only the four overlapping GLM source IDs were moved to 52–55; 20 GLM entries remain required, and no backend may alter the source. Missing Forge or Cranelift adapters must fail closed. The separate evidence validator binds each source repository's immutable numerical ID and does not accept step-free Actions runs. See `CI_EXECUTION_EVIDENCE.md`.
