# Define-scope / Newable reference vectors — source-free only

This branch contains no private compiler source bytes or Git blobs. It tests
independent policy invariants for modifier ordering, declaration nesting,
duplicate-modifier rejection and opt-in static Newable factory contracts.

The test workflow deliberately emits
`SOURCE_FREE_MODEL_ONLY_NOT_COMPILER_NOT_GUEST_NOT_AOT`.
Its results **must not** be attributed to the private formatter compiler,
Java/GraalVM guest runtime, LLVM, .NET, Wasm or Forge. This can establish
that test-org GitHub Actions runners admitted actual steps; it cannot
establish compiler correctness.

Do **not** mirror `ores-truffle-oreslang/oreslang-format` into this repo.
Formatter source governance denies all cross-repository source export until
the protected source main branch, exact approved private destination ID,
reviewed immutable plan, independent approval and source publisher guard
all pass. See the private source's security contract.

The source PRs remain in draft until independent, exact-head,
compiler-backed executed test evidence is available.
