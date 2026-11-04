# Backend adapter readiness

The parity runner deliberately separates the corpus from backend-specific launch mechanics. Every backend must receive the same source bytes; adapters may build or package them, but must not rewrite Oreslang source to fit a backend.

| Backend | Current compile/check surface | Current execution surface | Multi-file readiness |
|---|---|---|---|
| Java / GraalVM | `OresMain --check source.ores` via Maven or a built Oreslang CLI | `OresMain source.ores`; Native Image CLI can use the same source surface | Best current reference candidate; linker already has relative-file import support |
| LLVM native | `scripts/compile-aot.sh source.ores output` using built `oreslang-llvmc` + Clang | Execute emitted native binary | Native scalar/module support is the strongest direct compile/run adapter; unsupported features must reject |
| WASM | `scripts/compile-ores.mjs --compiler=... --source=... --output=...` using a pinned LLVM compiler | `scripts/run-guest.mjs --wasm=... --sha256=...` | Current bridge explicitly supports standalone scalar source and says relative module graphs need a future pinned module-graph gate |
| .NET Native AOT | `oresnet check`; `oresnet compile source.ores --out .../OresGenerated.dll` | Generated CIL must be linked into `Ores.Net.NativeHost` with `dotnet publish`, then the native host executes | Requires an adapter wrapper for compile → host publish → run; current scalar compiler does not establish whole-program import parity |
| Forge | `oresforge ir source.ores` admits a limited scalar profile; failure must classify unsupported syntax | `oresforge run` may interpret and print a scalar to stdout; a reviewed adapter with a true guest-execution boundary is required | No general cross-module or independent whole-program parity proof; mandatory compiler, not an alias |
| Cranelift JIT | Limited source frontend to CLIF; source-only and scalar support | Reviewed native Cranelift JIT executable when available | OresDB async/actors/multi-file WAL remain unsupported; a missing adapter is a failure, never a pass |
| JavaScript transpiler | `node bin/oreslang-jsc.mjs --check source.ores`; compile with `--out=/absolute/output.mjs` | Import generated ESM and call exported `main()` | Scalar programs are executable; actors/async/import-graph support must fail closed when unsupported |

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
Its results are exploratory, never six-backend parity proof.

### LLVM native

After building `oreslang-llvmc`:

```sh
export ORES_PARITY_LLVM_COMPILE='/abs/oreslang-source.llvm/scripts/compile-aot.sh {source} {artifact}'
export ORES_PARITY_LLVM_RUN='{artifact}'
```

**O0/O2 differential gate (pending compiler-backed execution):** The
[LLVM draft #1346](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1346)
introduces `ORESLANG_LLVM_AOT_OPT_LEVEL=0|2` for the Clang IR-to-native
stage (default `2`). Run the **same reviewed source SHA** twice with the two
values, separate artifacts and stable compiler/toolchain identities. This
changes Clang optimization, **not necessarily earlier Oreslang frontend
passes**. Its CI synthetic-IR script and local Clang smoke are
**host-pipeline only** and do not certify any AttoLang guest.

A source-free canary workflow is also on shared parity PR #23. Its missing
job steps, compared with stepful source-free canaries on
`gha-indie-worker/gha-indie-worker.rs`, isolate the issue to source-repo/org
runner admission or billing/policy. Do **not** transfer private source solely
because an unrelated canary runs elsewhere. Review and authorize the exact
source tree and destination independently before any new mirror.

The shared runner now resolves an extensionless relative import from
`.atto` to `.atto`, or from historical `.ores` to `.ores`, without silently
rewriting the source. This tests inventory and source-graph fingerprinting
only; separate compiler-backed `.atto` acceptance is required on every
toolchain.

The compile script fails closed if the source subset is unsupported and never falls back to an interpreter.

### JavaScript

The JS CLI publishes executable ESM atomically. A run adapter must import that module and call exported `main()`; simply invoking the generated module is not enough because it exports `main` rather than automatically calling it.

A scalar adapter can publish to `{artifact}.mjs` and use Node's module mode to import it. The run wrapper must translate an integer/BigInt main result into the process-status oracle used by legacy cases, while void-main checks rely on their `PARITY_OK|...` stdout assertions.

### WASM gap

The Wasm bridge is intentionally stronger than a plain "run this file" command: execution requires the SHA-256 of the exact emitted Wasm bytes. The parity runner now supports a post-compile `{artifact_sha256}` placeholder in
the **run** argv template. It hashes the opened, non-symlink, nonempty bounded
artifact bytes after the compile command exits successfully. On Unix it opens
with `O_NOFOLLOW` and `O_NONBLOCK`; missing, directory, symlink, oversized, and
changing artifacts fail closed. A Wasm run template **must** include the
placeholder: omission is `runtime_infrastructure_failure`, not a parity
result. This binding does not authenticate the compiler or prove a trusted
release. The Wasm runtime must independently check its received digest.

Configure a reviewed adapter (with an exact binary/source SHA manifest), for
example:

~~~sh
export ORES_PARITY_WASM_COMPILE='node /abs/wasm/scripts/compile-ores.mjs --compiler=/abs/oreslang-llvmc --source={source} --output={artifact}'
export ORES_PARITY_WASM_RUN='node /abs/wasm/scripts/run-guest.mjs --wasm={artifact} --sha256={artifact_sha256}'
~~~

The compiled Wasm artifact remains a core `() -> i64`-only guest; it cannot
implement Orestra actor/state/OCI behavior yet. A compiler failure or missing
backend remains a parity gap, even with a valid digest.

The current Wasm bridge also documents that relative source imports require a future pinned module-graph gate, so multi-file checks should presently report an unsupported parity gap rather than being flattened or rewritten.

### .NET Native AOT gap

`oresnet compile` emits `OresGenerated.dll`, not the final executable. The repository's verified path is:

1. `oresnet compile source.ores --out .../OresGenerated.dll`
2. `dotnet publish host/Ores.Net.NativeHost/Ores.Net.NativeHost.csproj ... -p:GeneratedAssembly=<absolute OresGenerated.dll>`
3. execute the resulting native `oreslang-app`.

That needs a dedicated adapter wrapper or a richer parity-runner pipeline. Do not treat successful CIL emission alone as runtime parity.

## Promotion rules

An adapter is executable evidence only when it records the exact compiler/runtime SHA, compiles the unchanged source graph, classifies negative diagnostics, and runs the produced program where the backend claims execution support. Compile-only support, an unsupported feature, a skipped test, or a missing runner is a parity gap rather than a pass.

### Cranelift explicit fail-closed admission

The runner now includes `cranelift` as a sixth **required compiler** backend alongside the independent `forge` compiler, with `ORES_PARITY_CRANELIFT_COMPILE` and `ORES_PARITY_CRANELIFT_RUN`. The six required compiler backends are Java, LLVM, WASM, .NET, Forge, and Cranelift. JavaScript is an optional seventh adapter selected only explicitly. An absent adapter is `backend_not_configured` in all 73 current cases, and exit 125 is `compile_infrastructure_failure`; neither grants parity. The scalar-only Cranelift JIT currently cannot claim ODB1 WAL/actor integration.

### Forge remains mandatory

`forge` is not an alias for `cranelift`. Do not satisfy Forge's proof with a Cranelift, LLVM or interpreter-only result. Use independently pinned compiler/runtime artifacts, and classify currently unsupported source as a parity gap (not `pass`). The default inventory enumerates 73 cases on each of the six required compilers (438 `not_run` cells until independently executed); JavaScript's optional 73-cell inventory is not in the required matrix.
