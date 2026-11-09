# Adapter fault-injection E2E boundary

The adapter_fault_injection_e2e.rs suite launches the actual Rust parity-runner
binary with synthetic compiler and guest subprocesses. It exercises manifest
loading, subprocess invocation, negative diagnostics, guest output validation,
exit classification and fail-closed report generation together.

Faults covered: unclassified compiler errors; extra guest output after a
valid-looking success prefix; exploratory execution without independent
validation; and infrastructure exit code 125. Unit tests also reject duplicate
success lines, bad/newline-less output, overflow, noncanonical checksums,
wrong names and invalid UTF-8.

Multi-file stdout must be exactly one line in the form
PARITY_OK|<case-name>|<canonical-signed-i64-checksum> followed by LF.
This strengthens transcript integrity but does not establish the expected
semantic checksum. Such cases remain unvalidated until an independent oracle
is reviewed.

Run with cargo test --manifest-path tools/parity-runner/Cargo.toml --locked.
These shell stand-ins do NOT execute an AttoLang compiler or certify guest
parity. Real certification requires an authorized exact-head runner, pinned
compiler/guest artifacts and a separately verified ORES_GUEST_EXECUTED receipt.

When two or more backends produce canonical output for the same multi-file
case, the runner also compares their complete stdout byte strings. Different
checksums now classify every candidate participant as
cross_backend_stdout_mismatch, even when each alone looks like a valid
success line. The cross-backend full-process scenario uses synthetic Java and
LLVM stand-ins with conflicting checksums and expects a hard failure.

Reviewed deterministic stdout goldens (still NOT_CERTIFIED without compiler/guest
execution) are pinned for order-of-operations=36, actor-system=15,
do-switch=126, do-match=126, recursion=176. Any canonical line carrying a
*different* checksum now reports wrong_stdout. Checks without an independently
reviewed oracle still permit canonical exploratory output but cannot be
promoted to verified until expected_stdout_exact is present. These goldens
are derived by manual review of the checked-in sources, not by trusting the
compiler under test. Their runtime assumptions remain subject to real guest
verification.

## Source graph mutation fence

The runner now checks the original entry SHA-256 and complete recursively
reachable import-graph fingerprint at three boundaries: before compiler
invocation, immediately after compiler invocation, and after guest execution.
Any drift or newly encountered symlink/non-file changes the case status to
`source_integrity_failure`, which is always a hard failure. In particular,
a compiler that mutates a transitive imported module cannot claim a passing
result based on the earlier pre-execution graph fingerprint. A focused unit
regression edits both a leaf import and the entry source to establish this
rejection. This fence is not an immutable source sandbox: a hostile compiler
could change and restore bytes between sampling boundaries. Future execution
needs immutable staged source graphs or OS-enforced read-only mounts.
