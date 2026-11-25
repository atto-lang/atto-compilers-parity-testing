# Cranelift native parity: currently blocked

The six required targets are Java, LLVM, Wasm, .NET NativeAOT, Forge, and Cranelift; JS remains optional.

Cranelift's shipped native JIT is an independent, narrow checked-i64 scalar compiler, not an actor/ADT/async/full-language compiler. The CLI offers `ir` (source to textual Cranelift IR) and `run` (native JIT prints scalar output). Neither produces a standalone artifact whose exit code is the guest return value. An unreviewed stdout-to-exit shim or hard-coded exit 42 would falsify parity. Until a reviewed native result adapter exists, attempted required execution must return `backend_not_configured` or another nonpass status.

Evidence validator pins `ores-truffle-oreslang/oreslang-source.cranelift` repository ID `1437273004` (do not confuse it with `oreslang-source.forge`).

## Release gates

1. Pin the exact Cranelift commit/tree, Rust toolchain/lockfile and executable SHA-256 before compiling guest source. Match byte-identical `.ores` fixture SHA-256 on all backends.
2. Implement and independently review a real JIT adapter with a checked numeric result, bounded stdout/stderr, nonzero error classification, and no interpreter or constant-result fallback.
3. Run 40–49 through actual Cranelift JIT, Java/GraalVM, LLVM machine code, Wasm and .NET NativeAOT and compare typed values and effects.
4. Keep all unsupported syntax fail-closed and preserve separate runner-admission, host test, compiler build, guest receipt, and reviewer gates. Source was not mirrored to any other organization.
