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
