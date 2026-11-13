# Atto actor critical-state three-backend admission — candidate V1

**STATUS: AUTHORED_NOT_EXECUTED.** This is an additive native parity
candidate, not part of the legacy 53-case `.ores` acceptance score or
a claim of successful compilation on any backend.

## Fixed source, targeted failures

The positive fixture `fixtures/two-actor-snapshot-positive.atto` is
byte-identical to the four broker candidate PRs (NATS #31, Kafka #34,
RMQ #25, Redis #42) at the heads recorded in `contract.json`:
Git blob `a6189d92c813426ecf54a09cf535c3874de45942`.
It uses `critical-write` to mutate sealed actor state, then a
**detached** `critical-read` scalar followed by a mailbox send only after
the state loan ends. Two actors must each produce the scalar 1. This is
only a *logical* actor-isolation fixture: no physical per-actor heap,
32-bit ref32 allocation or nonblocking broker I/O is proved by its output.

Two negative fixtures differ only by the intentionally invalid
`critical-read` span:
1. `deny-send-inside-read.atto` sends a borrowed actor-state field from
   inside the live critical scope. The compiler must issue a targeted
   ownership/effect diagnostic, not execute a send or fall back.
2. `deny-write-inside-read.atto` mutates protected state through a
   read-only loan. The compiler must reject before code emission.

The Rust conformance test at
`tools/parity-runner/tests/actor_critical_three_backend.rs` validates
the original Git blob, the two exact minimal mutations, the required
backend inventory and the contract's strict **NOT_EXECUTED** state.
These tests never invoke an Atto compiler and cannot be used to claim
Java/LLVM/.NET guest success. CI must execute them with real Rust
(`cargo test --manifest-path tools/parity-runner/Cargo.toml --test actor_critical_three_backend`).

## Required backend evidence

The **identical positive and negative bytes** must pass through all:
- Java/Graal: actual Atto production parser + type/borrow checker,
  executable guest and verified output, not a Java guest double.
- LLVM: native frontend to LLVM IR to Clang-built executable at O0/O2,
  plus independently executed sanitizers and expected diagnostics.
- .NET: production Atto parser/typed CIL to full NativeAOT executable,
  Linux and Windows where supported. Opt-in generated C# demo does
  not establish this production source path.

Evidence must pin the source/fixture blob and compiler source/tree
identity, real job/runner IDs and nonzero test steps, plus native
artifact digests and actual exit/output. A generic parse error does not
prove a targeted borrow rejection. Backend skip, retagging Java
results as LLVM/.NET, mocks, interpreter substitutions, zero-step
jobs and unreviewed source transfers are invalid.

## Deeper isolation and performance gates

A passing fixture is **necessary, not sufficient** for broker release.
Prove actor-owned logical stacks and physically backed checked 32-bit
arena-relative heaps with generation-fenced reclamation, immutable
sharing or exclusive writer/compatible-reader guards for zero-copy,
and bounded mailbox ownership with exact failed-send rollback.
`define supervisor` / `define pub supervisor` must use authenticated
CONTROL lanes, separate from DATA. Network and storage operations must
use true nonblocking readiness/completions; outsourcing a blocking
call to another thread is not equivalent.

Protocol workloads require independent Rust clients/oracles, durability
and ACK correctness, matching configuration/hardware and >=5
independent valid trials. Only actual native Atto median throughput
>=70% of the equivalent Rust baseline (or the consistently chosen
official Java Kafka baseline) can satisfy the performance objective.
This conformance candidate provides **no performance number**.
