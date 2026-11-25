# 35-program design (31 original + 4 candidate integer-boundary tests)

This corpus is a feature ladder, not 30 variations of arithmetic.

| Range | Purpose |
|---|---|
| 01-06 | scalar execution, calls, modules, named arguments |
| 07-12 | conditional, switch and match control-flow semantics |
| 13-17 | fail-closed diagnostics |
| 18-22 | structural data, tuples, ownership and closures |
| 23-27 | cleanup, exceptions, async and channels |
| 28-30 | actor lifecycle, lexical isolation and critical state |
| 31 | static select outer-delimiter parity (`{ ... }` vs `over ... end`) |
| 32 | signed i64 minimum plus maximum, no intermediate overflow |
| 33 | signed i64 maximum exact subtraction |
| 34 | exact near-maximum signed i64 multiplication and subtraction |
| 35 | signed multiplication, unary negation, associativity |

Java, LLVM, WASM, .NET and Forge are the five required backends for every case. The JavaScript transpiler is available only as an explicit, optional target. A backend that does not implement a case is a **parity gap**, not a skip/pass. The validation_status field records how strongly the source is grounded today; only independently executed and oracle-checked cases should later be promoted to validated.

Cases 27, 30 and 32–35 intentionally remain candidates because their exact source/runtime contracts still require executable reference validation. They are real stress programs, not placeholders.

Case 31 is a Java reference-test target. Backends that currently reject static select must report that as a parity gap; an explicit fail-closed diagnostic is better than accidental parsing or fake execution, but it does not count as feature parity.

## Multi-file whole-program layer

The `checks/` tree adds 16 positive programs and two negative syntax probes.
Positive `main.ores` files each import three unique local files and assert
their semantic expectations. The Rust runner fingerprints the recursively
reachable graph and records those files for each backend result.

The 53-case, five-required-backend audit matrix contains 265 records (and 53 optional JavaScript records if explicitly selected). A not-run record
is corpus inventory, not compiler/runtime conformance. Ordinary match/switch
suspension remains distinct from the rejected `nb match` / `nb switch`
spellings. The modern actor counter uses `handle_state` for mutation.

## Cranelift/signed-i64 additions (40–49)

Ten unvalidated candidate programs stress lossless signed-64 arithmetic above the f64 mantissa, signed-MIN/MAX boundaries, checked multiplication, nesting and named bitwise XOR. An independent BigInt oracle confirms 42 for each expression; no native Oreslang compiler has executed the new fixtures. Forge remains required, JavaScript stays optional, and Cranelift is added as a separate required sixth backend.
