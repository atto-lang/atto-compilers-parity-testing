# Verifying downloaded GitHub Actions artifacts (stacked follow-up)

The existing Rust metadata inspector returns a GitHub-API-reported
\`sha256:<64-hex>\` digest. This PR adds a separate, **offline** verifier of
the actual downloaded ZIP bytes against that digest and the trusted fixture set.

The verifier accepts **only two entries**:
- \`guest-receipt.json\`: a bounded strict JSON object, no unknown or duplicate
  fields, with repository/run/job/source/tree/backend/feature/status counts
- \`fixture-set.json\`: bounded strict JSON array whose SHA-256 must equal the
  independently configured \`case_set_sha256\`

The ZIP must be at most 8 MiB compressed; receipt at most 1 MiB and fixture-set
at most 4 MiB uncompressed. Paths, symlinks/directory entries, duplicate or extra
entries, unsupported/invalid status, host-only claims and stale run identities
are rejected.

**Trust boundaries:** Comparing the archive's SHA-256 to GitHub's reported
digest establishes byte consistency with the trusted API metadata only if
that metadata was fetched and authenticated separately. The contents of a
matching \`guest-receipt.json\` can still lie about what executed. Consequently
the verifier returns \`guest_execution_verified=false\` even for valid bytes.
Its fixture hash proves embedded fixture bytes, not that a guest consumed them.
Production release status remains \`NOT_CERTIFIED\`. Next gates must inspect
guest execution proof, immutable runner/workflow provenance, and backend behavior.

Use only Rust for parsing, ZIP decompression and hashing. Offline tests include
real compressed ZIP archives, hash mutations, zip-bomb bounds, archive path
traversal, duplicate fields and cross-field/identity failures. Do not silently
treat an artifact metadata digest or a self-authored receipt as sufficient.


## Combined read-only metadata + downloaded ZIP verification CLI

The GitHub inspector also accepts two **trusted local inputs**:

```sh
GH_TOKEN=... cargo +1.88.0 run --locked --bin atto_github_inspect -- \
  ./github-policy.json --artifact-zip ./downloaded-artifact.zip \
  --receipt-policy ./guest-receipt-policy.json
```

The GitHub policy remains the exact pinned source/run/tree/reviewer/job/step/artifact
policy described above. The guest-receipt policy requires `repository, run_id,
job_id, source_sha, tree_sha, backend, feature_id, case_set_sha256`
(the hashes are lowercase/uppercase hex; the string values must match the
independent GitHub metadata and the actual ZIP contents).

The downloaded ZIP must be obtained separately from the trusted GitHub Actions
artifact endpoint. The program **does not fetch an arbitrary archive URL**;
it hashes the provided bytes and compares the digest to the authenticated
GitHub API claim. A mismatched digest, receipt or fixture hash exits nonzero
with `NOT_CERTIFIED`. A matching archive reports byte/fixture
verification but STILL returns `guest_execution_verified=false` and
`NOT_CERTIFIED` because metadata and fixture receipts cannot prove
that the backend actually executed the tests.


## Direct authenticated ZIP download

Once the GitHub metadata verifier has accepted the exact run, job, tree,
independent review and protected-main checks, the inspector may download
the **specific artifact ID** directly:

```sh
GH_TOKEN=... cargo +1.88.0 run --locked --bin atto_github_inspect -- \
  ./github-policy.json --download-artifact \
  --receipt-policy ./guest-receipt-policy.json
```

It begins at the fixed api.github.com origin, uses authenticated GET and
follows at most two GitHub-issued **HTTPS-only** redirects (not a URL from
the report). Curl does not use `--location-trusted` so credentials are not
forwarded to another host. Download size is limited to 8 MiB, and the Rust
verifier independently rehashes the ZIP bytes against GitHub's reported SHA-256.
As before, guest execution remains **NOT_CERTIFIED** until independent guest
runtime evidence is validated.
