# Multi-file cross-compiler checks

Each lowercase folder is a whole-program parity test. `main.ores` imports three local source files (some folders also contain a negative probe), performs explicit semantic assertions, and emits one stable success line:

```
PARITY_OK|<check-name>|<checksum>
```

A compiler/backend only passes when it consumes the same source graph, finishes without a compiler/runtime crash, all internal assertions pass, and stdout matches the other backends. Run the same corpus at multiple optimization levels and execution modes. Do not rewrite a test per backend.

## Initial matrix

- `checks/import-cycles/`
- `checks/order-of-operations/`
- `checks/fuzz-seeds/`
- `checks/mutex-locking/`
- `checks/task-blocks/`
- `checks/actor-system/`
- `checks/do-switch/`
- `checks/do-match/`
- `checks/nb-match/`
- `checks/nb-switch/`
- `checks/named-parameters/`
- `checks/optional-parameters/`
- `checks/anonymous-functions/`
- `checks/recursion/`
- `checks/classes-traits-casting/`

`nb-match` and `nb-switch` deserve special treatment: the current grammar permits `do match` and `do switch`, but not `nb match` or `nb switch`. Ordinary value-producing `match` already has tested suspension semantics, so that folder asserts await in the subject, guard, and arm and separately includes a compile-reject `nb match` probe. `nb-switch` follows the same model.

`optional-parameters` is intentionally a spec-convergence probe for the desired distinction between `T?` (Option shorthand) and `name?` (omittable parameter). Keep it frontier until the canonical grammar and every backend agree.

## Differential-testing references

- GCC torture tests: https://gcc.gnu.org/onlinedocs/gccint/C-Tests.html
- LLVM testing guide: https://llvm.org/docs/TestingGuide.html
- LLVM whole-program test suite: https://llvm.org/docs/TestSuiteGuide.html
- Rust compiletest/UI tests: https://rustc-dev-guide.rust-lang.org/tests/compiletest.html
- Csmith differential random testing: https://github.com/csmith-project/csmith
- YARPGen multi-file checksum differential testing: https://github.com/intel/yarpgen
- Equivalence Modulo Inputs (EMI): https://web.cs.ucdavis.edu/~su/emi-project/

The most useful pattern for Oreslang is a blend: tiny GCC/LLVM-style regression programs, Rust-style run-pass/compile-fail expectations, Csmith/YARPGen differential outputs, and EMI/metamorphic variants that must remain equivalent on fixed inputs.
