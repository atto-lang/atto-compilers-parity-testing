# 30-program design

This corpus is a feature ladder, not 30 variations of arithmetic.

| Range | Purpose |
|---|---|
| 01-06 | scalar execution, calls, modules, named arguments |
| 07-12 | conditional, switch and match control-flow semantics |
| 13-17 | fail-closed diagnostics |
| 18-22 | structural data, tuples, ownership and closures |
| 23-27 | cleanup, exceptions, async and channels |
| 28-30 | actor lifecycle, lexical isolation and critical state |

All five backends are targets for every case. A backend that does not implement a case is a **parity gap**, not a skip/pass. The `validation_status` field records how strongly the source is grounded today; only independently executed and oracle-checked cases should later be promoted to validated.

Cases 27 and 30 intentionally remain candidates because their exact source/runtime contracts still require executable reference validation. They are real stress programs, not placeholders.

## Multi-file whole-program layer

The `checks/` tree adds 16 positive whole-program checks and 2 negative syntax probes. Each positive `main.ores` imports exactly three unique local source files and asserts its semantic oracle internally.

This layer targets interactions single-file fixtures miss: import SCCs, precedence/associativity, deterministic fuzz/metamorphic invariants, mutexes, task/async boundaries, actors, procedural match/switch, named and optional parameter convergence, parameter-order syntax, lambdas, mutual recursion, and nominal/interface/trait casting.

The runner fingerprints the recursively reachable source graph and records its files in every backend result.
