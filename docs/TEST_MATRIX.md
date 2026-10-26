# 31-program design

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

All five backends are targets for every case. A backend that does not implement a case is a **parity gap**, not a skip/pass. The validation_status field records how strongly the source is grounded today; only independently executed and oracle-checked cases should later be promoted to validated.

Cases 27 and 30 intentionally remain candidates because their exact source/runtime contracts still require executable reference validation. They are real stress programs, not placeholders.

Case 31 is a validated Java reference target. Backends that currently reject static select must report that as a parity gap; an explicit fail-closed diagnostic is better than accidental parsing or fake execution, but it does not count as feature parity.
