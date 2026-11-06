# Execution and parity gate

The checked-in corpus is fail-closed. The default CI audits 40 legacy programs (including four unvalidated signed-i64 boundary candidates and five router-derived `.atto` candidates), 16 positive multi-file checks and 2 negative probes, yielding 58 logical cases and 348 required backend/case pairs across Java, LLVM, WASM, .NET, Forge and Cranelift. JavaScript is an optional seventh target (`--backend js`), not a substitute for Forge. It makes no compiler-parity claim unless a case is explicitly validated and an exact backend adapter executes it.

## Rust-only tooling

Oreslang organization policy prohibits Python tooling. The parity harness is implemented in Rust under `tools/parity-runner`. Do not introduce Python, Python actions, or Python helper scripts.

## Exploratory execution

Each backend uses two command templates:

- `ORES_PARITY_<BACKEND>_COMPILE`
- `ORES_PARITY_<BACKEND>_RUN`

Tokens: `{source}`, `{artifact}`, `{workdir}`. Templates are split into argv without a shell. Supported backend names are LLVM, JAVA, WASM, DOTNET, FORGE, CRANELIFT and optional JS. The default report includes only the six required backends.

Example:

```sh
export ORES_PARITY_JS_COMPILE='node /absolute/oreslang-source.js/bin/oreslang-jsc.mjs {source} --out={artifact}.mjs'
export ORES_PARITY_JS_RUN='node /absolute/run-esm-main.mjs {artifact}.mjs'
cargo run --manifest-path tools/parity-runner/Cargo.toml -- --backend js --execute --allow-unvalidated --output js-exploratory.json
```

Exploratory matching is **not** a parity pass. An executed run returns success only
when **every selected result** is `pass` or `expected_reject` with validated
semantics; exploratory matches, unknown statuses and infrastructure failures exit
nonzero, even when a report is successfully written. A no-`--execute` corpus
inventory may exit successfully, but records only `not_run` cases.

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

The pinned Java reference adapter previously attempted the 49-case corpus exploratorily; it has **not** yet run the four new candidates (32–35); see
`JAVA_REFERENCE.md`. No cases are promoted by adapter execution alone. The first
pinned reference run exposed corpus/compiler syntax and type-checking gaps;
these must be resolved against canonical contracts without backend-specific rewrites.

Case 31 is the Java-reference `do select ... over ... end` target. Explicit fail-closed rejection on LLVM/Wasm/.NET/JS is useful capability evidence, but remains a parity gap rather than a pass until those backends execute the same source semantics.

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
not five-backend execution proof. The `intentional_capability_reject` status
is a fail-closed gap, never a passing record.

Cranelift is also a required independent target, alongside LLVM, Java, Wasm, .NET and Forge; JavaScript remains optional. The 58-case default audit now yields 348 non-executed cells. Cranelift uses ORES_PARITY_CRANELIFT_COMPILE and ORES_PARITY_CRANELIFT_RUN. Its scalar JIT has no reviewed separate artifact adapter for this corpus; unavailable or unsupported source remains a gap, never a pass.
