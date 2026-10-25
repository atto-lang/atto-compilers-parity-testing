# Execution and parity gate

The checked-in corpus is fail-closed. The default CI audits source presence and enumerates 48 logical cases across five backend slots (240 records). It makes no compiler-parity claim unless a case is explicitly validated and an exact backend adapter executes it.

## Rust-only tooling

Oreslang organization policy prohibits Python tooling. The parity harness is implemented in Rust under `tools/parity-runner`.

The runner audits 30 reference-grounded single-file cases, 16 positive multi-file checks, and 2 explicit negative syntax probes. For each positive multi-file check it recursively follows relative imports, requires at least a four-file reachable graph, validates exactly three unique direct imports from `main.ores`, and records a SHA-256 over the complete reachable graph.

Positive multi-file execution expects exit 0 plus `PARITY_OK|<check-name>|`. Negative probes remain fail-closed until the backend adapter also classifies the expected diagnostic.

## Exploratory execution

Backend command templates are `ORES_PARITY_<BACKEND>_COMPILE` and `ORES_PARITY_<BACKEND>_RUN`, with `{source}`, `{artifact}`, and `{workdir}` tokens. Supported backend names are LLVM, JAVA, WASM, DOTNET, and JS.

Exploratory matching is **not** a parity pass. `task-blocks`, `named-parameters`, and `optional-parameters` are explicit convergence probes and do not claim current-main acceptance.

## Runner evidence

The parity repository's source-org Actions jobs have repeatedly failed before any step was created: `runner_id=0`, blank runner name, `steps=[]`. That is infrastructure failure, not corpus evidence.

A source-free canary in `sonus-auris/sonus-auris.infra` succeeded on run `40219779480`, hosted runner `1000027553`. No Oreslang compiler or parity source was copied cross-organization. Alternate capacity therefore exists, but private-source transfer still requires explicit authorization.

## Known debt

Legacy cases 27 and 30 intentionally remain candidates pending executable reference validation. The next execution stage needs exact backend adapters, diagnostic classification, optimization-matrix runs, scheduler fuzzing, and memory/ownership sanitizer coverage.
