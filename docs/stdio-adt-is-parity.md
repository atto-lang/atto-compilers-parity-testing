# Checked stdin/stdout and nominal on-is ADT patterns: differential parity

**Status:** target contract plus executable backend-test harness, **not** an assertion of guest support.

Sources: [atto-spec #32](https://github.com/atto-lang/atto-spec/pull/32) (language/stdio-channel-endpoints-v1.md and language/adt-is-variant-patterns-v1.md); [atto-platform-conformance #50](https://github.com/atto-lang/atto-platform-conformance/pull/50); compiler implementation candidate [Rust #257](https://github.com/atto-lang/atto-source.rs/pull/257).

## Required syntax

~~~atto
on is Ok<_> -> match << input over
  on is Ok<line> -> match line over
    on is Some<item> -> item,
    on None -> "",
  end,
  on is Err<error> -> "",
end
~~~

This is a **match arm fragment** inside a larger string-producing return-match. The first Ok<_> tests the checked write, the second Ok<line> binds an Option<string> after stdin read, and Some<item> binds string. None represents EOF; the empty-string fallback is an application decision, not an EOF sentinel or implicit unwrap. The angle brackets are binding syntax, never a generic type test or arbitrary extractor.

## Actual compiler/guest gate

Run the portable suite through **tools/run-stdio-conformance.mjs** with Node.js 22+ and an explicit local configuration. The runner requires exact Git SHA pins for both the real compiler checkout and atto-platform-conformance checkout. Each backend supplies its own *actual* compiler and guest launch commands. No command is guessed; no shell, mock compiler, host reference model or JS string evaluation counts as a guest.

Example configuration **shape only** (replace every placeholder; not a verified command for any backend):

~~~json
{
  "backend": "llvm",
  "toolchain": "PIN_EXACT_COMPILER_AND_RUNTIME_VERSIONS",
  "compilerRepo": {
    "path": "/absolute/checkout/atto-source.llvm",
    "sha": "0000000000000000000000000000000000000000"
  },
  "suiteRepo": {
    "path": "/absolute/checkout/atto-platform-conformance",
    "sha": "0000000000000000000000000000000000000000"
  },
  "compile": ["/actual/compiler", "{source}", "-o", "{artifact}"],
  "execute": ["/actual/guest-launcher", "{artifact}"],
  "timeoutMs": 15000
}
~~~

Reproduction:

~~~sh
node --test tests/stdio-conformance-runner.test.mjs
cargo test --locked --test stdio_adt_parity
node tools/run-stdio-conformance.mjs /absolute/path/to/config.json
~~~

Each guest-pass case requires real compilation to a nonempty artifact and exact stdin bytes / exit / stdout / stderr for all seven recorded scenarios. Every compile-fail case requires nonzero compiler exit with its exact diagnostic and *no guest artifact*. Source-compile-only fixtures are not guest successes. An unproven stream/readch race is rejected, not skipped.

The HDL profile requires explicit E_UNSUPPORTED_PROFILE refusal and no artifact; native CLI profiles must not use the HDL exception. All files and compiler checkout heads must match pinned SHA, with per-case and artifact SHA-256 digests. The runner restricts subprocess environment/FD inheritance, bounds output and time, and always writes releaseGate=NOT_VERIFIED; independent CI run/job steps and external review are still required.

## Backend obligations

| Backend | Required evidence | Current state |
| --- | --- | --- |
| Java/Graal | Nonparking channel stdio, capability broker, JVM/Native Image guests | NOT_EXECUTED |
| LLVM | Native async/reactor, checked ownership, executable CLI | NOT_EXECUTED |
| Forge | Independent compiler/runtime source-built CLI | NOT_EXECUTED |
| .NET | CIL and Native AOT checked stdio guest | NOT_EXECUTED |
| Rust source compiler | Versioned authenticated JSPI checked guest | NOT_EXECUTED |
| Independent Wasm | Independently executed Wasm guest, injected explicit ports | NOT_EXECUTED |
| JS | Checked Worker endpoints (not untrusted TenantVM sandbox) | NOT_EXECUTED |
| Cranelift | Native typed endpoint lowering and guest | NOT_EXECUTED |
| Atto self-hosted | Bootstrap compiler and execution evidence | NOT_EXECUTED |
| HDL | E_UNSUPPORTED_PROFILE, no artifact | NOT_EXECUTED |
| Realtime | Bounded nonparking or explicit unsupported profile | NOT_EXECUTED |

Do not promote PASS from parser-only acceptance, metadata tests, host models, zero-step runner failures or stale-SHA receipts.
