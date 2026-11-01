# 49-program design

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
| 32-39 | immutable-by-default bindings, explicit mutation, casts and diagnostics |
| 40-46 | `suspends` and critical-body semantics, incompatible effects |
| 47-49 | removed-operator rejection (`%`, `^`, `^^`) |

All six backends (Java, LLVM, Wasm, Cranelift, .NET, JS) are targets for every case. A backend that does not implement a case is a **parity gap**, not a skip/pass. The validation_status field records how strongly the source is grounded today; only independently executed and oracle-checked cases should later be promoted to validated.

Cases 27 and 30 intentionally remain candidates because their exact source/runtime contracts still require executable reference validation. They are real stress programs, not placeholders.

Case 31 is a Java reference-test target. Backends that currently reject static select must report that as a parity gap; an explicit fail-closed diagnostic is better than accidental parsing or fake execution, but it does not count as feature parity.

## Multi-file whole-program layer

The `checks/` tree adds 16 positive programs and two negative syntax probes.
Positive `main.ores` files each import three unique local files and assert
their semantic expectations. The Rust runner fingerprints the recursively
reachable graph and records those files for each backend result.

The 67-case, six-backend audit matrix contains 402 records. A not-run record
is corpus inventory, not compiler/runtime conformance. Ordinary match/switch
suspension remains distinct from the rejected `nb match` / `nb switch`
spellings. The modern actor counter uses `handle_state` for mutation.

New cases 32–49 are **candidates**, not compiler-verified. Their diagnostic categories remain intentionally unmapped until a real compiler's output is independently reviewed. Legacy tests 25/27/28/30 now use `suspends` and are also candidate-status pending compiler proof.
