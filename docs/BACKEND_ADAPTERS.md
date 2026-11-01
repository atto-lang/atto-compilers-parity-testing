# Backend adapter readiness

The parity runner deliberately separates the corpus from backend-specific launch mechanics. Every backend must receive the same source bytes; adapters may build or package them, but must not rewrite Oreslang source to fit a backend.

| Backend | Current compile/check surface | Current execution surface | Multi-file readiness |
|---|---|---|---|
| Java / GraalVM | `OresMain --check source.ores` via Maven or a built Oreslang CLI | `OresMain source.ores`; Native Image CLI can use the same source surface | Best current reference candidate; linker already has relative-file import support |
| Cranelift native JIT | `orescranelift ir source.ores` (a real Cranelift IR build, not a saved artifact) | `orescranelift run source.ores` runs independently compiled native JIT; prints signed-i64 result plus LF, exits 0 | One main function, checked scalar i64 subset only; multi-file imports unsupported |\n| LLVM native | `scripts/compile-aot.sh source.ores output` using built `oreslang-llvmc` + Clang | Execute emitted native binary | Native scalar/module support is the strongest direct compile/run adapter; unsupported features must reject |
| WASM | `scripts/compile-ores.mjs --compiler=... --source=... --output=...` using a pinned LLVM compiler | `scripts/run-guest.mjs --wasm=... --sha256=...` | Current bridge explicitly supports standalone scalar source and says relative module graphs need a future pinned module-graph gate |
| .NET Native AOT | `oresnet check`; `oresnet compile source.ores --out .../OresGenerated.dll` | Generated CIL must be linked into `Ores.Net.NativeHost` with `dotnet publish`, then the native host executes | Requires an adapter wrapper for compile → host publish → run; current scalar compiler does not establish whole-program import parity |
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
Its results are exploratory, never five-backend parity proof.

### LLVM native

After building `oreslang-llvmc`:

```sh
export ORES_PARITY_LLVM_COMPILE='/abs/oreslang-source.llvm/scripts/compile-aot.sh {source} {artifact}'
export ORES_PARITY_LLVM_RUN='{artifact}'
```

The compile script fails closed if the source subset is unsupported and never falls back to an interpreter.

### JavaScript

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

## Running unchanged real ORM source across all six backends

The opt-in `--orm-root` and `--orm-head` pair uses a separately authorized local
checkout of `postgres-orm.ores`. No private source is fetched, mirrored, exported
or rewritten in the parity repository. The exact 40-hex HEAD, Git object IDs,
staged/unstaged status and bounded source file identities are checked.

Four native Oreslang guest cases compile the *actual* original modules:
`tests/schema-plans.ores`, `tests/schema-adversarial.ores`,
`tests/schema-invariants.ores` (all import `src/schema.ores`), and
`tests/plans.ores` (imports `src/pg.ores`). The runner fingerprints each
entire admitted two-file graph and demands exact guest stdout, including LF.

Example on an approved machine with a pinned ORM checkout:

```sh
cargo run --locked --manifest-path tools/parity-runner/Cargo.toml -- \
  --orm-root /abs/approved/postgres-orm.ores \
  --orm-head FULL_40_HEX_CHECKED_OUT_COMMIT \
  --backend llvm --case-id orm-sql-plans \
  --output orm-llvm-inventory.json
```

Only after real compiler adapters are configured add `--execute
--allow-unvalidated`. Source-only inventory reports `not_run`, never parity.
A successful unvalidated run reports an exploratory match until an independent
semantic oracle review. The default remains 55 cases × 6 backends = 330
cells; an opt-in ORM root adds four cases for 59 × 6 = 354 cells, 24 extra
backend/guest candidates. Real Java, LLVM, Cranelift, WASM and .NET compilers
must all receive the unchanged original Oreslang bytes.

Known blockers: Cranelift currently implements a narrow scalar JIT, not the
full ORM nominal/Array/Result/import and actor runtime. Wasm needs complete
module import support, an admitted guest stdout ABI and pinned Wasm artifact.
.NET needs whole-program class/constructor lowering plus NativeAOT execution.
Java and LLVM also need exact-head compiled/running guest receipts.
All unsupported capabilities are gaps. Full database parity further requires
portable async database capabilities, parameter/OID codecs, actor-owned
connections, tenant RLS, cancellation and transactions, TypeSpec/JSON parity,
and DPM SQL catalog replay. No ORM migration apply authority is granted.


### Concrete pinned ORM source candidate (not a compiler pass)

Source ORM commit `89ec3ed01de695d7f7f24602500041f659b294e2`,
Git tree `60da9563917ca166f00c9a09ef3346196fec378e`.
The six genuine `.ores` blobs at that exact immutable commit are:

| File | Git blob SHA-1 |
| --- | --- |
| `src/schema.ores` | `5cda802a2b257928a8f102ec1fc127eb61f263b6` |
| `src/pg.ores` | `d9cfe1c1f3cc1ed14ee61cf80e648dd22291d924` |
| `tests/schema-plans.ores` | `c73e36b0d4622c43fa8506c3100fe33b2213d2c9` |
| `tests/schema-adversarial.ores` | `0beb839b9908b6c4995e1522bbfa03e6c1e04a99` |
| `tests/schema-invariants.ores` | `624121a254440116423e220e89c9d7db09b162c0` |
| `tests/plans.ores` | `58482d494207f657c85f56805c719d409181aedd` |

These are Git blob identities, **not** native execution results. The runner
independently reads the on-disk bytes and verifies their actual Git object
identities before trying any compiler. Follow-on ORM changes require another
exact-head and reviewed graph, not silent replacement of these blobs.
