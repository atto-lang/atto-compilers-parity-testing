# Read-only GitHub evidence inspector — stacked follow-up

Usage:
```sh
GH_TOKEN=... cargo +1.88.0 run --locked --bin atto_github_inspect -- ./trusted-policy.json
```

The trusted policy file specifies repository, pull_number, run_id, source_sha, tree_sha, job_name, step_name, artifact_name and reviewer. Paths are constructed only from validated repository names and numeric identifiers. Never fetch an untrusted report-provided URL.

The Rust verifier reads workflow run metadata, exact commit/tree SHA, PR head, specific successful job+step+runner, exact-head independent approval, main branch protection/required checks, and unexpired named artifact metadata with a GitHub-reported SHA-256 digest. The curl transport uses GET only, a fixed https://api.github.com origin, a credential delivered on stdin instead of argv, no redirects, and strict JSON decoding.

**Always NOT_CERTIFIED.** A GitHub-reported digest is a claim, not an independently hashed downloaded archive. A green job step name does not demonstrate actual guest execution. This follow-up does not download artifact bytes, validate fixture hashes inside artifacts, establish backend isolation, or certify releases. Missing API permissions (including private-repo branch protection), missing reviews, incomplete pages, stale SHAs and absent artifacts fail closed. Rust offline tests use stub API responses, not real source repositories. Root contract PR #1 stays untouched until independent approval.
