# Java / Rust semantic-conflict differential candidate v1

**Status: authored candidate, no compiler/guest execution certified.** This is an additive AttoLang corpus, not a rewrite or promotion of the historical 53-case Oreslang parity suite.

## Scope and language authority

This candidate compares the same, untouched .atto fixture bytes against three independently pinned implementations:

- atto-lang/atto-source.java (Java/GraalVM).
- atto-lang/atto-source.rs root compiler (Rust to browser WebAssembly).
- atto-lang/atto-source.rs/native (historical Oreslang Rust/Cranelift native compiler imported as a subtree).

These are separate targets. Rust/Wasm is not the Cranelift native backend. Normative language behavior comes from atto-lang/atto-spec, not whichever compiler happens to accept a fixture. All ten fixtures have their exact SHA-256 in contracts/java-rust-conflicts-v1/manifest.json and each backend starts as NOT_EXECUTED.

## Ten cases and known contradictions

| Test ID | Status | Required observation |
| --- | --- | --- |
| caret-return-int-xor | Pending spec #143 | A leading ^ returns; integer ^^ computes XOR, yielding 42 |
| infix-caret-reject | Pending spec #143 | A single infix ^ is invalid; targeted E_OPERATOR_CARET |
| constant-overflow-reject | Checked arithmetic target | Constant signed-i64 overflow cannot be folded to a wrapped value |
| runtime-overflow-panic | Checked arithmetic target | Dynamic signed-i64 overflow cannot wrap or return normally |
| trap-preserves-result | Normative trap-result v1 | trap on an already Result-valued operand must not add a Result or Option layer |
| trap-catches-ordinary-throw | Normative trap-result v1 | Ordinary source throw produces Err(original Error); raise/panic/host failures are not caught |
| lazy-bool-guards | Normative boolean rules | && and || short-circuit; guest stdout is exactly 42 + newline |
| legacy-comma-evaluation | DECISION REQUIRED | Historical Java short-circuit vs browser Rust eager condition |
| void-result-binding | DECISION REQUIRED | Historical Java accepts void binding vs browser Rust rejects |
| implicit-int-float | DECISION REQUIRED | Historical Java permits selected implicit numeric widening vs Rust browser rejects |

The three decision-required probes are **observation-only** (including when a backend rejects them at compile time); no parser or guest behavior can be called compliant until the language spec adjudicates them. The caret/XOR cases are still candidate syntax while spec PR #143 remains unmerged. Source acceptance, inferred diagnostics, and runtime outputs are hypotheses until executed against pinned real compilers.

## How to use

Metadata-only locally, without any compiler:

    node --test tests/java-rust-differential.test.mjs
    node tools/run-java-rust-differential.mjs --audit

This checks file SHA-256 and the 30 NOT_EXECUTED cells. A successful Node test does **not** claim any AttoLang guest execution.

A trusted operator on a host holding its own appropriately authorized compiler files can use an explicit, private JSON adapter:

    node tools/run-java-rust-differential.mjs --execute --backend rust-wasm --adapter /absolute/private/adapter.json --output /absolute/private/observations.json

Required adapter fields: backend, compiler_commit (40 hex), compiler_tree (40 hex), compiler_executable (absolute regular file), compiler_sha256 (64 hex), guest_runner_executable (absolute regular file), guest_runner_sha256 (64 hex), compile_argv, run_argv, value_mode (exit_code for small 0–255 values, or stdout_i64 for exact signed 64-bit decimal strings), plus optional unsupported_cases and unsupported_needles indexed by fixture ID. Unsupported requires an actual nonzero compiler rejection with the named diagnostic code and **no files** in the temporary output directory; merely declaring a case unsupported does not produce an UNSUPPORTED record. Arbitrary diagnostic substrings are not admitted. Commands are arrays of argv strings with {source}, {artifact}, {workdir} substitutions; compilation must mention source and artifact. Caller-supplied source rewrites, shell argument expansion, missing artifacts, generic non-targeted negative diagnostics, and unchecked output are never a valid parity pass.

Each stage has a 15-second timeout and a 1 MiB captured-output limit; compilation runs in a new temporary directory and the compiled artifact is SHA-256 checked before and after its guest runner. Negative tests require a nonzero compiler rejection, an exact targeted diagnostic-category token, and **no output files**. Local candidate matches are labeled OBSERVED_UNVERIFIED and the report always has guest_parity_certified=false. Explicit unsupported is UNSUPPORTED; wrong output is MISMATCH; host faults are INFRASTRUCTURE_FAILURE. Even perfect local observations are not independently attested execution, and an execution report exits **3** (never green) when all provisional observations match; mismatches/infrastructure errors exit 2. Metadata-only audits may exit 0.

Before any release promotion, independently bind source head and full tree, fixture Git blob, binary and artifact digests, reviewed runner/wrapper blob, actual target/toolchain, exact guest command, nonzero substantive workflow steps, run/attempt/job identity, normalized outputs and targeted diagnostic categories. Local adapter metadata cannot prove these claims. This repository does not export private source across organizations or install any compiler. Use atto-platform-conformance evidence governance for accepted receipts.

## Related PRs and overlap disposition

Independent of this repository's HTTP/FS #19, stdio/ADT #23 and evidence verifier #6/#8, all of which cover different interfaces. Backend feature work remains in atto-source.rs #439 (overflow), #331/#275 (trap), #411 (caret/XOR), atto-source.java #66 (overflow), #67 (host-fault trapping), and competing Java #70/#72 (caret/XOR). Spec authority: atto-spec #143 and language/trap-result-v1.md. Conformance candidate: atto-platform-conformance #68. **Do not duplicate implementation PRs or merge the two overlapping Java caret PRs without reconciling their behavior and security tests.**

The workflow java-rust-differential-audit.yml deliberately executes **metadata and harness tests only**. An empty/zero-step CI job is NOT_EXECUTED, not a successful guest validation. No PR merge is authorized by this corpus.

Signed-i64 oracles may be canonical decimal strings to preserve all 64 bits (up to 9223372036854775807 and down to -9223372036854775808). A guest stdout_i64 result requires a zero exit code; overflow/trap markers must be emitted on stderr and accompanied by nonzero exit. A compiler or guest crash is infrastructure failure, never a checked-overflow success.

## Classifier hardening checkpoint (2026-11-20)

The exact diagnostic code required for an UNSUPPORTED observation must be `E_UNSUPPORTED` or a namespaced `E_UNSUPPORTED_...` code. A generic parser, syntax, type or overflow failure cannot be relabeled unsupported by a user-supplied adapter. An undecided-language observation may record an actual compile-time rejection only when no partial artifact files were written; successful compilation without the requested artifact is a MISMATCH, never OBSERVATION_ONLY. Compilers and guests remain unexecuted until true source-owned or independently authorized runner receipts exist.

## Native Rust/Wasm guest-host adapter: concrete bounded path (2026-11-20)

The upstream source interfaces were inspected directly. Browser compiler atto-lang/atto-source.rs/src/bin/attoc.rs accepts the actual CLI invocation: attoc source.atto -o output.wasm. That invocation is now the only allowed compile_argv shape for the rust-wasm backend. It must use an absolute pinned compiler executable; arbitrary source wrappers and argument variants are rejected.

The new tools/guest/run-wasm-i64.mjs is a real WebAssembly.Module and WebAssembly.Instance host, not a synthetic Atto compiler. It rejects imports, invalid/symlinked or oversized artifacts, missing exported main, non-i64 returns, and nonzero exported panic_code (if present). It prints the signed 64-bit main result as canonical decimal plus a newline. Unknown Wasm traps fail with WASM_GUEST_TRAP_UNCLASSIFIED; they must never be relabeled as INTEGER_OVERFLOW without backend-specific evidence. This restricted host deliberately does not implement WASI, stdout imports or actors, so it cannot pass every corpus fixture today.

Required backend-specific JSON adapter contract: backend=rust-wasm; compiler_commit and compiler_tree from the exact source Git commit; compiler_executable and compiler_sha256 for the compiled native attoc binary; guest_runner_executable and guest_runner_sha256 for Node; guest_runner_script and guest_runner_script_sha256 for this checked-in Wasm host; compile_argv of [compiler_executable, {source}, -o, {artifact}]; run_argv of [guest_runner_executable, guest_runner_script, {artifact}]; and value_mode=stdout_i64. All paths to local executables, scripts, adapter JSON and output must be absolute. Script file bytes are independently SHA-256 rechecked on every execute invocation. Source commit/tree fields are claims until independently bound to the actual built compiler and exact-run receipt.

An operator with an independently authorized local compiler can build atto-lang/atto-source.rs with cargo build --locked --release --bin attoc, verify git rev-parse HEAD and HEAD^{tree}, hash the built attoc, Node and this checked-in guest host, create the private adapter outside the repository, then invoke node tools/run-java-rust-differential.mjs --execute --backend rust-wasm --adapter /absolute/private/adapter.json. The outcome is only provisional LOCAL_OBSERVATION_NOT_ATTESTATION and always exits nonzero.

Tests/run-wasm-i64.test.mjs and tests/validate-wasm-adapter.test.mjs are source-free tests of the host and config shape. They use synthetic tiny WebAssembly bytes, including a real imported-module rejection, signed i64, symlink rejection and a non-attributed Wasm trap. Their local Node 22 execution passed eight tests (including the oracle-scope regression); exact Git blobs of both modules and test files matched the locally tested files. This verifies host behavior, NOT native attoc compilation or any current-head guest comparison.

Rust/Cranelift is distinct: native/src/main.rs emits an object with oresc source.atto artifact.o and requires a trusted linker and executable guest driver. Java/GraalVM uses the OresMain/Truffle launcher and requires its own reviewed compile/evaluate bridge. Neither target is implemented by this Wasm host. The entire thirty-cell matrix remains NOT_EXECUTED pending actual pinned compiler binaries, governed execution and nonzero-step CI.

**Oracle-scope correction:** the restricted Wasm host's decimal output represents the return value of its exported main function, **not guest stdout**. It also lacks a source-typed overflow/trap diagnostic ABI. Therefore the differential runner explicitly reports INFRASTRUCTURE_FAILURE (reason WASM_HOST_CANNOT_VERIFY_STDOUT_OR_TYPED_TRAP) for rust-wasm fixtures of kind stdout or trap, rather than accidentally scoring a matching printed return value as guest I/O or an unclassified Wasm trap as language overflow. Native/other backends are unaffected. A future reviewed runtime-ABI adapter may add those observations; it must not silently repurpose this host.
