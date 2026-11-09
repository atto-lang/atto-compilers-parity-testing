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
