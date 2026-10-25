# Execution and parity gate

The checked-in corpus remains **unvalidated**. The default CI only audits source presence and enumerates the 150 backend/case pairs. It makes no compiler claims.

## Local exploratory runs

Set `ORES_PARITY_LLVM_COMPILE` and `ORES_PARITY_LLVM_RUN` to backend-specific commands. Templates use `{source}`, `{artifact}` and `{workdir}`, are tokenized with `shlex`, and run without a shell. Configure JAVA, WASM, DOTNET and JS similarly.

Example for a hypothetical wrapper:
```sh
export ORES_PARITY_LLVM_COMPILE='python3 /absolute/path/llvm-compile-wrapper.py {source} {artifact}'
export ORES_PARITY_LLVM_RUN='/absolute/path/run-llvm {artifact}'
python3 scripts/parity_runner.py --execute --allow-unvalidated --output local-exploratory.json
```

**Do not** treat exploratory matching as passing parity. Compiler commands and runtimes must be verified on exact source revisions. All five backend adapters remain to be implemented and verified.

## Promotion gate

A case may advance to `validated` only after its canonical syntax is confirmed, independent semantic oracle reviewed, and all non-placeholder features genuinely exercised. For negative tests, verify diagnostic category as well as nonzero compile status. Change `expected_kind` only against an authoritative compiler specification. Exit-code testing (42) cannot prove stdout, nondeterministic event ordering, memory safety or diagnostic parity on its own.

For funded-runner fallback, mirror only an identical authorized source tree into an explicitly approved test organization; compare Git tree/blob identities, prohibit secrets in untrusted workflows, and report exact build compiler/runtime SHAs. A cloned private tree is a disclosure to the destination organization; require authorization first.

## Known debt

24, 25, 29 and 30 are coverage placeholders. 23 needs a real actor-isolation violation. Several parser-frontier examples likely need grammar adjustments. Cases 03/04 require suppression of invalid eager evaluation while also passing constant-folding checks. Future iterations need expected traces, stdout comparison, timeout/rejection diagnostics, optimization-matrix execution and memory/ownership sanitizer tests.


## Multi-file checks

The runner now audits both corpora in one fail-closed matrix:

- 30 legacy single-file candidates from `manifest.json`
- 15 multi-file checks from `checks/manifest.json`
- 2 explicit negative syntax probes (`nb-match`, `nb-switch`)
- 47 logical cases total
- 5 backend slots each = 235 records

For each positive multi-file check, the audit recursively follows relative Oreslang imports, requires at least a four-file reachable source graph, and records a deterministic SHA-256 over the complete reachable graph. The direct entry-point import count is checked against `expectation.json`; multiple declaration imports from the same source file count as one imported file.

Positive multi-file execution expects process exit 0 and a stable stdout prefix `PARITY_OK|<check-name>|`. Negative probes remain fail-closed: a nonzero compiler exit is recorded as `rejection_unclassified` until the backend adapter also classifies the expected diagnostic.

## Current Actions admission evidence — 2026-10-25

The parity repository's `Corpus audit` run for commit `23dc3447fc398474aed1c6fccf5940b2a0a94f92` failed before any step was created: the job has `runner_id=0`, a blank runner name and `steps=[]`. This is runner admission/infrastructure failure, not a corpus result.

A source-free canary in private repository `sonus-auris/sonus-auris.infra` succeeded the same day on hosted runner `1000027553` with all three canary steps completed. No Oreslang compiler source and no parity repository source were copied to that organization. This proves alternate hosted-runner capacity exists but does not authorize cross-organization transfer of private source.
