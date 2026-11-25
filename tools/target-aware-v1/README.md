# Trusted-local candidate execution adapters

These adapters are additive to the legacy runner. They currently admit exactly two lanes: `js-esm` (the real `bin/oreslang-jsc.mjs` transpiler with explicit scalar or program-v2 profile) and `rs-wasm` (the real native `attoc` CLI). They compile the identical, unmodified [checked-i64-quotient.atto](https://github.com/atto-lang/atto-platform-conformance/blob/main/fixtures/numeric-guest-v1/checked-i64-quotient.atto) source pinned by Git blob `d7b61afd0fe074395b092aa4abaa08de4961602e`. Its expected signed result is 42. The numeric registry now pins all eleven cases from upstream manifest blob `8be4f89e994a4ace92f5de0f6d4b678ab60c1797`. Set `case_id` to select the exact source; omission defaults to the quotient case. Set `profile: "js-program-v2"` explicitly on the JS lane to admit all eleven cases, including three closed error identities. The default `strict-i64` retains the older CLI/profile and rejects negative cases; Rust still rejects the structured ordinary-error oracle rather than equating terminal panic with throw. A selected fixture can still fail compilation. All other lanes remain unimplemented.

Run on a trusted, clean, reviewed compiler checkout; use an independently built and authenticated Rust binary. This is not a hostile-code sandbox, a reproducible build verifier, or a certificate issuer. The JS compiler entrypoint digest is not the digest of its transitive dependency graph; the clean tracked tree and separately reviewed toolchain/dependencies are also required. Ignored files, Git configuration, toolchain origin, imported dependencies and authenticity of supplied pin values are not fully verified. Never use untrusted artifacts or give the runner access to secrets.

Example configuration (replace each placeholder with independently obtained actual values):

```json
{
  "lane": "rs-wasm",
  "root": "/absolute/canonical/atto-source.rs",
  "commit": "<40 hex head>",
  "tree": "<40 hex root tree>",
  "attoc": "/absolute/canonical/target/release/attoc",
  "compiler_sha256": "<64 hex executable digest>",
  "node_sha256": "<64 hex resolved Node executable digest>",
  "source": "/absolute/canonical/checked-i64-quotient.atto"
}
```

For JS set lane to `js-esm`, root to the JS checkout and compiler_sha256 to the SHA-256 of its tracked CLI entrypoint. Omit attoc. Set `profile` to `js-program-v2` for the full shared numeric corpus. The local error candidate requires a frozen RangeError with own data properties for name, code and matching candidate ABI number (65537 overflow, 65538 zero divisor), thrown during main execution. The observer cannot authenticate the compiler-private WeakSet brand; a trusted artifact can forge this shape, so the result is explicitly not a certified portable Error receipt. The guest artifact is emitted in a fresh temporary directory; execution uses a separate time-limited Node child process and compares the full BigInt return value, never the process exit code. Wasm rejects host imports and requires the panic ABI. Output and artifact sizes are bounded at the read/process-output boundary; this does not enforce filesystem or process memory quotas.

```sh
node tools/target-aware-v1/local-adapter.mjs config.json
node --test tools/target-aware-v1/validate.test.mjs tools/target-aware-v1/local-adapter.test.mjs tools/target-aware-v1/local-process.test.mjs
```

Local results are candidate observations only, with unclassified compile/guest failures distinguished from matches. A matching result is not an authenticated CI receipt, and the adapter's local exit status is not a release gate. Negative compile diagnostics, structured ordinary-error oracles for non-JS lanes, actual CI provenance, and clean reproducible build/engine identity remain separate required work. The new workflow runs host contract tests only; no compiler guest result may be inferred from it.

## Wave 9 verification

At candidate commit `bffd3d9471af62ec6835eb81a18cec129d082364`, six exact Git blobs (adapter, validator, both registries, adapter and subprocess tests) were reconstructed and hash-verified locally. Node 22.16.0 executed 16/16 test groups successfully. The subprocess tests use synthetic compiler artifacts, not the actual Atto compiler. Actual JS compiler numeric tests ran independently in [JS Actions 45688932217](https://github.com/atto-lang/atto-source.js/actions/runs/45688932217) at source head `472087eb1900135bd469ff86a8e5c3a39202fbf3`, tree `4bc6ea66cf791eaf6d5b7383743a81bbac8fe3f6`: dedicated numeric suite 25/25 passed, full suite 1327/1331. No cross-backend pass is claimed.
