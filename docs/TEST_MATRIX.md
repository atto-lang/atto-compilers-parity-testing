# 55-program reconciled design (31 original, 4 checked-i64, 20 GLM)

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
| 32–35 | canonical checked-i64 boundaries and multiplication tests (preserved from main) |
| 36–48 and 52–55 | fixed-size GLM/MoE integer control, routing, capacity and training-step fencing |
| 49–51 | fixed-gate GLM/MoE floating-point prediction, analytic derivative and SGD loss |

Java, LLVM, WASM, .NET NativeAOT, Forge, and Cranelift are the six required compilers for every case; JavaScript is optional. A backend that does not implement a case is a **parity gap**, not a skip/pass. The validation_status field records how strongly the source is grounded today; only independently executed and oracle-checked cases should later be promoted to validated.

Cases 27, 30 and 32–35 intentionally remain candidates because their exact source/runtime contracts still require executable reference validation. They are real stress programs, not placeholders.

Case 31 is a Java reference-test target. Backends that currently reject static select must report that as a parity gap; an explicit fail-closed diagnostic is better than accidental parsing or fake execution, but it does not count as feature parity.

## Multi-file whole-program layer

The `checks/` tree adds 16 positive programs and two negative syntax probes.
Positive `main.ores` files each import three unique local files and assert
their semantic expectations. The Rust runner fingerprints the recursively
reachable graph and records those files for each backend result.

The 73-case, six-required-backend audit matrix contains 438 records, with another 73 optional JavaScript inventory cells. A not-run record
is corpus inventory, not compiler/runtime conformance. Ordinary match/switch
suspension remains distinct from the rejected `nb match` / `nb switch`
spellings. The modern actor counter uses `handle_state` for mutation.

## GLM numerical and generation boundary

The 17 integer control cases and three float cases are all **candidate**
status until compiled by the Java/Graal reference and independently by the
LLVM, Wasm, .NET, Forge, and Cranelift backends. The 20 new source files are identical for
every backend; no fallback to integer-surrogate floating behavior is valid.

The top-two counted-load case intentionally proves only aggregate
arithmetic; it does **not** validate distinct per-token expert IDs, atomic
reservations, or concurrent tenant isolation. The three floating-point
cases use exactly representable binary fractions to avoid unnecessary
rounding disputes, but this is not an excuse to report unsupported float
lowering as a pass. A real compiler and guest receipt remains mandatory.

## Canonical ID preservation

Main's signed i64 programs retain IDs 32–35 and exact source paths. Four GLM integer programs have relocated to IDs 52–55, retaining names, semantic oracles and native source bodies apart from the ID banner comment. No backend-specific source rewriting is permitted. This reconciliation is a manifest/inventory migration, **not an executed compiler result**.
