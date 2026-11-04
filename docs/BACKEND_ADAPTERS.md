# Backend adapter readiness

The parity runner deliberately separates the corpus from backend-specific launch mechanics. Every backend must receive the same source bytes; adapters may build or package them, but must not rewrite Oreslang source to fit a backend.

| Backend | Current compile/check surface | Current execution surface | Multi-file readiness |
|---|---|---|---|
| Java / GraalVM | `OresMain --check source.ores` via Maven or a built Oreslang CLI | `OresMain source.ores`; Native Image CLI can use the same source surface | Best current reference candidate; linker already has relative-file import support |
| LLVM native | `scripts/compile-aot.sh source.ores output` using built `oreslang-llvmc` + Clang | Execute emitted native binary | Native scalar/module support is the strongest direct compile/run adapter; unsupported features must reject |
| WASM | `scripts/compile-ores.mjs --compiler=... --source=... --output=...` using a pinned LLVM compiler | `scripts/run-guest.mjs --wasm=... --sha256=...` | Current bridge explicitly supports standalone scalar source and says relative module graphs need a future pinned module-graph gate |
| .NET Native AOT | `oresnet check`; `oresnet compile source.ores --out .../OresGenerated.dll` | Generated CIL must be linked into `Ores.Net.NativeHost` with `dotnet publish`, then the native host executes | Requires an adapter wrapper for compile → host publish → run; current scalar compiler does not establish whole-program import parity |
| Forge (required) | `oresforge ir source.ores` checks the currently admitted scalar subset (and interprets it for overflow validation) | `oresforge run source.ores` returns **stdout number and process exit 0**, so a reviewed conversion adapter is required; `emit-asm` supports only narrow native subsets | Independent C++20 bootstrap; no general multi-file Oreslang compile/run adapter or complete native proof |
| JavaScript transpiler (optional) | `node bin/oreslang-jsc.mjs --check source.ores`; compile with `--out=/absolute/output.mjs` | Import generated ESM and call exported `main()` | Scalar programs are executable; actors/async/import-graph support must fail closed when unsupported |

## Runner templates

The Rust runner accepts two argv templates per backend:

- `ORES_PARITY_<BACKEND>_COMPILE`
- `ORES_PARITY_<BACKEND>_RUN`

Available placeholders are `{source}`, `{artifact}`, and `{workdir}`. Templates are tokenized without a shell.

### Java reference CLI

Use the Rust `java-reference` adapter described in [JAVA_REFERENCE.md](JAVA_REFERENCE.md).
It builds a clean pinned compiler checkout, fingerprints the launcher inputs,
and invokes the standalone Java CLI directly. Maven is used only during preparation;
`exec:java` does not preserve the guest program's process exit-status boundary.

```sh
cargo build --release --locked --manifest-path tools/parity-runner/Cargo.toml --bins
tools/parity-runner/target/release/java-reference prepare /abs/java-checkout COMMIT /abs/jdk /abs/java-bundle.json
tools/parity-runner/target/release/java-reference execute /abs/java-bundle.json /abs/java-results.json
```

This runs the JVM reference with `--mode=aot` (guest JIT disabled), not a Native Image executable.
Its results are exploratory, never five-backend parity proof.

### LLVM native

After building `oreslang-llvmc`:

```sh
export ORES_PARITY_LLVM_COMPILE='/abs/oreslang-source.llvm/scripts/compile-aot.sh {source} {artifact}'
export ORES_PARITY_LLVM_RUN='{artifact}'
```

The compile script fails closed if the source subset is unsupported and never falls back to an interpreter.

### Forge compiler: mandatory target, adapter pending

Forge is an independent compiler/backend and **must not** be represented by JS or LLVM evidence. The current C++20 bootstrap supports checked signed-i64 arithmetic, but its normal `run` command prints the interpreted value and exits 0. This is not the process-exit contract used by the legacy parity corpus. A reviewed Forge adapter must validate the compiler binary SHA and compilation profile, preserve genuine negative diagnostic output, and translate only a successfully executed, exact integer result into the legacy exit oracle. It must distinguish interpreter evidence from real emitted native code, refuse unsupported/multi-file graphs, and fail closed. No such adapter or native five-backend execution proof is claimed by this change.

Forge's current compiler provides specific diagnostic needles for out-of-range i64 literals, duplicate functions and call arity. These are indexed in `diagnostics.json`; no generic `expected integer` error is accepted as `unknown_identifier`, and unsupported-feature diagnostics remain gaps. The current `15-arity-reject` fixture also uses type-first parameters, which Forge's name-first scalar bootstrap does not admit. A different earlier syntax error must not be misclassified as an arity pass.

### JavaScript (optional)

The JS CLI publishes executable ESM atomically. A run adapter must import that module and call exported `main()`; simply invoking the generated module is not enough because it exports `main` rather than automatically calling it.

A scalar adapter can publish to `{artifact}.mjs` and use Node's module mode to import it. The run wrapper must translate an integer/BigInt main result into the process-status oracle used by legacy cases, while void-main checks rely on their `PARITY_OK|...` stdout assertions.

### WASM gap

The Wasm bridge is intentionally stronger than a plain "run this file" command: execution requires the SHA-256 of the exact emitted Wasm bytes. The current parity template vocabulary does not expose a post-compile artifact digest to the run command.

Do **not** weaken `run-guest.mjs` by omitting its SHA pin. Close this gap either by:

1. extending the Rust runner with an explicitly selected artifact path and `{artifact_sha256}` placeholder, or
2. using a reviewed backend wrapper that compiles, hashes, admits and executes the Wasm artifact as one fail-closed operation.

The current Wasm bridge also documents that relative source imports require a future pinned module-graph gate, so multi-file checks should presently report an unsupported parity gap rather than being flattened or rewritten.

### .NET Native AOT gap

`oresnet compile` emits `OresGenerated.dll`, not the final executable. The repository's verified path is:

1. `oresnet compile source.ores --out .../OresGenerated.dll`
2. `dotnet publish host/Ores.Net.NativeHost/Ores.Net.NativeHost.csproj ... -p:GeneratedAssembly=<absolute OresGenerated.dll>`
3. execute the resulting native `oreslang-app`.

That needs a dedicated adapter wrapper or a richer parity-runner pipeline. Do not treat successful CIL emission alone as runtime parity.

## Promotion rules

An adapter is executable evidence only when it records the exact compiler/runtime SHA, compiles the unchanged source graph, classifies negative diagnostics, and runs the produced program where the backend claims execution support. Compile-only support, an unsupported feature, a skipped test, or a missing runner is a parity gap rather than a pass.
