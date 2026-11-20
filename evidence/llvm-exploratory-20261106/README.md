# LLVM native exploratory run: 2026-11-06

`report.json` is a local run of all 49 cases with
`ores-parity-runner --execute --allow-unvalidated --backend llvm` against
`oreslang-source.llvm` branch `feat/llvm-parity-then-write-import-fnc-20261103`
at `d031b1681` (PR ores-truffle-oreslang/oreslang-source.llvm#1443, not yet merged). The merged PR head `4d8df03d5` gives identical counts. The compiler was built against Homebrew
LLVM 21.1.8 and executables were linked with the same Clang on macOS arm64.
Adapter: `scripts/compile-aot.sh {source} {artifact}` / `{artifact}`.

Result: **13 exploratory runtime matches, 6 classified exploratory rejections,
28 compile failures, 2 unclassified rejections.** Before that branch, `main`
gave 9 matches and 0 classified rejections. No case is promoted. This is not
cross-backend parity proof.

| Case | Java ref 2026-10-28 | LLVM before | LLVM now |
| --- | --- | --- | --- |
| recursion | match | compile_failure | **match** |
| import-cycles | compile_failure | compile_failure | **match** |
| order-of-operations | compile_failure | compile_failure | **match** |
| named-parameters | compile_failure | compile_failure | **match** |
| overflow-literal-reject | expected_reject | unclassified | **expected_reject** |
| arity-reject | expected_reject | unclassified | **expected_reject** |
| unknown-identifier-reject | expected_reject | unclassified | **expected_reject** |
| duplicate-function-reject | expected_reject | unclassified | **expected_reject** |
| nb-match-negative-1 | expected_reject | unclassified | **expected_reject** |
| nb-switch-negative-1 | expected_reject | unclassified | **expected_reject** |

The new `diagnostics.json` LLVM patterns cover only rejections that happen for
the case's intended reason. Two stay unclassified on purpose:

- `int-division-return-reject`: LLVM rejects float division as unsupported,
  not the int-return type mismatch the case targets.
- `actor-lexical-isolation-reject`: LLVM rejects a field-less actor before it
  reaches the isolation rule.

Largest remaining LLVM gaps where Java matches: `switch`/`match` statements
and expressions (5 cases), untyped `val` locals (`inferred-struct`), tuples,
closures, async/channels/select, and actor lifecycle.
