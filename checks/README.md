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
- `checks/parameter-syntax/`
- `checks/optional-parameters/`
- `checks/anonymous-functions/`
- `checks/recursion/`
- `checks/classes-traits-casting/`

`nb-match` and `nb-switch` deserve special treatment: the current grammar permits `do match` and `do switch`, but not `nb match` or `nb switch`. Ordinary value-producing `match` already has tested suspension semantics, so that folder asserts await in the subject, guard, and arm and separately includes a compile-reject `nb match` probe. `nb-switch` follows the same model.

`parameter-syntax` separately validates type-first (`string x`) versus name-first (`x: string`) declaration forms and mixed typed lambda parameter order, so it is not conflated with named call arguments.

`optional-parameters` is intentionally a spec-convergence probe for the desired distinction between `T?` (Option shorthand) and `name?` (omittable parameter). Keep it frontier until the canonical grammar and every backend agree.

## Actor frontier (November 2026)

- `checks/actor-mailbox-sequencing/`: three queued messages must commit through
  actor-owned `handle_state(critical ...)` and produce the ordered sequence
  1, 2, 3 before actor retirement.
- `checks/actor-message-fencing/`: stale-generation and foreign-tenant
  **application messages** must be rejected without modifying actor-owned
  state; one current message commits before retirement.

These are two **authored** multi-file Oreslang source graphs with three direct
imports each, not executed evidence. They do not cover supervisor restart,
mailbox backpressure, non-reentrancy across suspension, physical per-tenant
heaps, or an IsolateVM tenant authorization boundary.

Execution targets are Java/GraalVM, LLVM, WebAssembly/Wasmtime, .NET NativeAOT,
Cranelift and JavaScript. **Cranelift is scalar-only at stage 0**: an actor
program should produce a narrow unsupported-feature diagnostic, not a PASS.
Other backends are `NOT_RUN` until exact source graph, compiler SHA-256,
trusted runner steps, guest stdout and independent review exist. An unsupported
feature is a parity gap, not grounds to lower the source contract.
Do not move private source to another GitHub organization without authorization.

## Differential-testing references

- GCC torture tests: https://gcc.gnu.org/onlinedocs/gccint/C-Tests.html
- LLVM testing guide: https://llvm.org/docs/TestingGuide.html
- LLVM whole-program test suite: https://llvm.org/docs/TestSuiteGuide.html
- Rust compiletest/UI tests: https://rustc-dev-guide.rust-lang.org/tests/compiletest.html
- Csmith differential random testing: https://github.com/csmith-project/csmith
- YARPGen multi-file checksum differential testing: https://github.com/intel/yarpgen
- Equivalence Modulo Inputs (EMI): https://web.cs.ucdavis.edu/~su/emi-project/

The most useful pattern for Oreslang is a blend: tiny GCC/LLVM-style regression programs, Rust-style run-pass/compile-fail expectations, Csmith/YARPGen differential outputs, and EMI/metamorphic variants that must remain equivalent on fixed inputs.
