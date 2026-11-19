#!/usr/bin/env bash
# Read-only export of legacy Oreslang GitHub collaboration metadata for ALL 83 repos.
# Uses existing gh/git authentication; writes only to a private LOCAL folder.
# It does NOT transfer, recreate, close, comment on or modify any GitHub object.
set -euo pipefail
umask 077

fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
require() { command -v "$1" >/dev/null 2>&1 || fail "Missing executable: $1"; }
require gh
require git
require jq

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
manifest="${script_dir}/oreslang-atto-org-wide-repositories-2026-11-19.tsv"
[[ -f "$manifest" ]] || fail "Missing source map: $manifest"
[[ "${1:-}" != "" ]] || fail "Usage: $0 /absolute/path/to/NEW-PRIVATE-export-directory"
destination="$1"
[[ "$destination" = /* ]] || fail 'Choose an absolute output directory.'
[[ ! -e "$destination" ]] || fail "Output already exists: $destination (refusing overwrite)"
gh auth status >/dev/null 2>&1 || fail 'Existing GitHub CLI authentication is required.'
mkdir -m 700 -p -- "$destination"
printf '# Offline Oreslang collaboration archive\n# Export date: %s\n# 83 archived source repositories, read-only.\n' "$(date -u +%FT%TZ)" > "$destination/README.txt"
cp "$manifest" "$destination/repository-mapping.tsv"

# gh api --paginate emits one JSON array per page; jq --slurp merges them.
# No search API page-size caps or lossy excerpts.
export_array() {
  local endpoint="$1" outfile="$2"
  gh api --method GET --paginate -H 'Accept: application/vnd.github+json' "$endpoint" \
    | jq -s 'add // []' > "${outfile}.tmp"
  jq -e 'type == "array"' "${outfile}.tmp" >/dev/null || fail "Invalid array from $endpoint"
  mv -- "${outfile}.tmp" "$outfile"
}

processed=0
while IFS= read -r source_repo; do
  [[ -n "$source_repo" ]] || continue
  [[ "$source_repo" =~ ^ores-truffle-oreslang/[A-Za-z0-9_.-]+$ ]] || fail "Unexpected source name: $source_repo"
  name="${source_repo#*/}"
  out="$destination/$name"
  mkdir -m 700 -- "$out" "$out/pull_reviews"
  printf 'Exporting %s ...\n' "$source_repo"

  gh api "repos/$source_repo" > "$out/repository.json"
  export_array "repos/$source_repo/issues?state=all&per_page=100" "$out/issues-and-pr-issues.json"
  jq '[.[] | select(has("pull_request") | not)]' "$out/issues-and-pr-issues.json" > "$out/issues.json"
  export_array "repos/$source_repo/pulls?state=all&per_page=100" "$out/pull-requests.json"
  export_array "repos/$source_repo/issues/comments?per_page=100" "$out/issue-and-pr-conversation-comments.json"
  export_array "repos/$source_repo/pulls/comments?per_page=100" "$out/pr-inline-review-comments.json"
  export_array "repos/$source_repo/issues/events?per_page=100" "$out/issue-events.json"
  export_array "repos/$source_repo/labels?per_page=100" "$out/labels.json"
  export_array "repos/$source_repo/milestones?state=all&per_page=100" "$out/milestones.json"

  # Every PR's review objects, including approvals and dismissed reviews.
  while IFS= read -r number; do
    [[ "$number" =~ ^[0-9]+$ ]] || fail "Invalid PR number for $source_repo"
    export_array "repos/$source_repo/pulls/$number/reviews?per_page=100" "$out/pull_reviews/$number.json"
  done < <(jq -r '.[].number' "$out/pull-requests.json")

  # Original branches, tags and public GitHub PR refs: SHA metadata only.
  # The Git history migration itself MUST use an authorized Git pack fetch/push.
  git ls-remote --refs "https://github.com/$source_repo.git" \
    'refs/heads/*' 'refs/tags/*' 'refs/pull/*/head' 'refs/pull/*/merge' \
    > "$out/remote-git-refs.tsv"

  jq -n --arg repo "$source_repo" \
    --argjson issues "$(jq 'length' "$out/issues.json")" \
    --argjson prs "$(jq 'length' "$out/pull-requests.json")" \
    --argjson comments "$(jq 'length' "$out/issue-and-pr-conversation-comments.json")" \
    '{repository:$repo,issue_count:$issues,pr_count:$prs,conversation_comment_count:$comments}' \
    > "$out/counts.json"

  processed=$((processed+1))
done < <(tail -n +2 "$manifest" | cut -f1)

[[ "$processed" -eq 83 ]] || fail "Expected 83 source repositories, found $processed"
if command -v shasum >/dev/null 2>&1; then
  find "$destination" -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 shasum -a 256 > "$destination/SHA256SUMS"
elif command -v sha256sum >/dev/null 2>&1; then
  find "$destination" -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > "$destination/SHA256SUMS"
else
  fail 'No SHA-256 tool for verifiable metadata receipts.'
fi
printf 'Exported %s repositories to %s; no GitHub writes performed.\n' "$processed" "$destination"
printf 'WARNING: Keep the output private; it contains private issues, PRs and discussion metadata.\n'
