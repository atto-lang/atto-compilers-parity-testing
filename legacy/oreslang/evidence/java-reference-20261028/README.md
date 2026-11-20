# Java reference baseline: 2026-10-28

`report.json` is actual local execution of all 49 cases against compiler commit
`3210aeff07597035a4ed815ebbdf39720a3266c1`, using Temurin JDK 25.0.4.1 on macOS arm64.
The corpus/adapter implementation commit is recorded in `identity.json`.
The report binds the unchanged source graphs, diagnostics, exit statuses,
stdout/stderr bytes, and timings. The identity receipt hashes that report and
records compiler/runtime and launcher artifact pins. Absolute paths describe
the machine that performed the run; they are not portable adapter settings.

Result: **23 exploratory runtime matches, eight classified exploratory negative
matches, 18 compile failures**. Required execution exited 1. No cases were
promoted and this is not cross-backend parity proof. See
[`docs/JAVA_REFERENCE.md`](../../docs/JAVA_REFERENCE.md) for reproduction and gaps.

The Java compiler was built with its own tests skipped. The parity runner and
adapter have a separate ten-test regression suite. Hosted corpus-audit evidence,
when linked in the PR, validates tooling and inventory, not this Java guest run.
