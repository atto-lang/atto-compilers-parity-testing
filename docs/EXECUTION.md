# Execution and parity gate

The checked-in corpus is fail-closed. The default CI audits source presence and enumerates 48 logical cases across five backend slots (240 records). It makes no compiler-parity claim unless a case is explicitly validated and an exact backend adapter executes it.

## Rust-only tooling

Oreslang organization policy prohibits Python tooling. The parity harness is implemented in Rust under `tools/parity-runner`. Do not introduce Python, Python actions, or Python helper scripts.

The runner audits both corpora:

- 30 legacy single-file candidates from `manifest.json`
- 16 multi-file checks from `checks/manifest.json`
- 2 explicit negative syntax probes (`nb-match`, `nb-switch`)
- 48 logical cases total
- 5 backend slots each = 240 records

For each positive multi-file check, the runner recursively follows relative Oreslang imports, requires at least a four-file reachable source graph, validates the declared count of unique directly imported files, and records a deterministic SHA-256 over the complete reachable graph. Multiple declaration imports from one source file count as one imported file.

Positive multi-file execution expects process exit 0 and a stable stdout prefix `PARITY_OK|<check-name>|`. Negative probes remain fail-closed: a nonzero compiler exit is recorded as `rejection_unclassified` until a backend adapter also classifies the expected diagnostic.

## Exploratory execution

Each backend uses two command templates:

- `ORES_PARITY_<BACKEND>_COMPILE`
- `ORES_PARITY_<BACKEND>_RUN`

Tokens: `{source}`, `{artifact}`, `{workdir}`. Templates are split into argv without a shell. Supported backend names are LLVM, JAVA, WASM, DOTNET and JS.

Example:

```sh
export ORES_PARITY_JS_COMPILE='node /absolute/oreslang-source.js/bin/oreslang-jsc.mjs {source} --out={artifact}.mjs'
export ORES_PARITY_JS_RUN='node /absolute/run-esm-main.mjs {artifact}.mjs'
cargo run --manifest-path tools/parity-runner/Cargo.toml -- --execute --allow-unvalidated --output js-exploratory.json
```

Exploratory matching is **not** a parity pass.

## Promotion gate

A case becomes `validated` only after canonical syntax is confirmed against the reference frontend, its semantic oracle is independently reviewed, and the named feature is genuinely exercised. Negative tests require a diagnostic category, not merely nonzero exit status. Unsupported features, skips, timeouts, missing runners and infrastructure failures never count as passes.

`task-blocks`, `named-parameters`, and `optional-parameters` are currently explicit convergence probes: their expectation files describe the intended language contract but do not claim current-main acceptance.

For funded-runner fallback, mirror only an identical authorized Git tree into an approved test organization, verify tree/blob identity, prohibit secrets in untrusted workflows, and record exact compiler/runtime SHAs. Mirroring a private tree discloses it to the destination organization, so authorization is required.

## Current Actions admission evidence — 2026-10-25

The parity repository's source-org `Corpus audit` has repeatedly failed before any step was created, with `runner_id=0`, a blank runner name and `steps=[]`. This is runner admission/infrastructure failure, not a corpus result.

A source-free canary in private repository `sonus-auris/sonus-auris.infra` succeeded on run `40219779480`, hosted runner `1000027553`, with all three canary steps completed. No Oreslang compiler source and no parity repository source were copied to that organization. This proves alternate hosted-runner capacity exists but does not authorize cross-organization transfer of private source.

## Known debt

Legacy cases 24, 25, 29 and 30 remain coverage placeholders and legacy case 23 needs a precise actor-isolation violation. Several frontier cases need canonical grammar repair. The next execution stage needs exact backend adapters, diagnostic categorization for negative tests, optimization-matrix runs, scheduler fuzzing and memory/ownership sanitizer coverage.
