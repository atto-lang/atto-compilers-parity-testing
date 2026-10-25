# Oreslang compiler parity testing

30 **provisional** source programs for Java/GraalVM, LLVM, WASM, .NET and JavaScript transpilation. None has been executed here. All programs are common inputs (no backend-specific rewrites). They are intended as a starting corpus, not proof of conformance.

## Status and caveats

- `core`: syntax patterns grounded in existing Java/LLVM fixtures, but not compiler-verified in this repository.
- `frontier`: candidate syntax/behavior needing parser validation and adaptation to canonical Oreslang grammar.
- `negative`: expected compile rejection, never a runnable positive case.
- Cases **24, 25, 29, 30** are *scaffolds*, NOT real critical-section, backpressure, serialization or end-to-end tests. Upgrade them before reporting coverage.
- Case 23's negative oracle is provisional and must be replaced with an exact actor lexical-isolation violation. Do not report it as a validated rejection test.
- Case 01 returns 42; case 17 returns 42 (LIFO 1*10+2+30); all positive expected results 42. Do not confuse process exit code with stdout.
- Case 03 must be tested with a compiler that preserves laziness; the division branch may be rejected statically if constant-folded.

## Protocol

For each test/backend capture source SHA, compiler revision, compiler command, feature capability, compile outcome, normalized diagnostic category, process exit status, stdout bytes, stderr bytes, and timeout. Compare to independent golden oracle and other backends. Never treat a skip or an absent backend runner as a pass. Run optimized/unoptimized configurations; run deterministically with fixed seed and bounded time. Separate unsupported features from wrong-code, compiler crash, runtime crash, and infrastructure failure.

## Research basis

- Csmith differential compiler testing: https://github.com/csmith-project/csmith
- Equivalence modulo inputs (EMI): https://www.microsoft.com/en-us/research/publication/compiler-validation-via-equivalence-modulo-inputs/
- WebAssembly official spec/test suite: https://github.com/WebAssembly/spec
- Metamorphic relations: https://varylang.com/docs/vast/metamorphic-testing/

## Next step

Validate all 30 against canonical formatter/parser. Replace provisional syntax and scaffolds, add exact negative oracles, then implement five backend adapters. No CI execution claims are made by this commit.
