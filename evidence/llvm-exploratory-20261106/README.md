# LLVM native exploratory run: 2026-11-06

`report.json` is a local run of all 53 cases with
`ores-parity-runner --execute --allow-unvalidated --backend llvm` against
`oreslang-source.llvm` branch `feat/llvm-parity-then-write-import-fnc-20261103`
at `284c6b33d` (PR ores-truffle-oreslang/oreslang-source.llvm#1443, not yet merged). The compiler was built against Homebrew
LLVM 21.1.8 and executables were linked with the same Clang on macOS arm64.
Adapter: `scripts/compile-aot.sh {source} {artifact}` / `{artifact}`.

Result: **17 exploratory runtime matches, 6 classified exploratory rejections,
28 compile failures, 2 unclassified rejections** on 53 cases. On the original
49 cases that is 13 matches and 6 classified rejections, up from 9 and 0 on
LLVM `main`. The four cases added to the corpus since (`checked-wide-multiply`,
`checked-negative-products`, `signed-min-boundary`, `signed-max-boundary`) all
match. No case is promoted. This is not
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
