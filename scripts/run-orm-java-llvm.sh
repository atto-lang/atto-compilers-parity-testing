#!/usr/bin/env bash
# Real immutable ORM source, Java/GraalVM check+run, LLVM AOT compile+run.
# No source mirroring, fake compilers or host-reference guest substitutions.
set -euo pipefail
fail() { printf 'ORM parity PRECHECK_FAILED: %s\n' "$*" >&2; exit 69; }
[[ $# -eq 6 ]] || fail 'usage: run-orm-java-llvm.sh ORM_ROOT ORM_HEAD JAVA_ROOT LLVM_ROOT LLVM_HEAD NEW_EVIDENCE_DIR'
for cmd in git cargo realpath; do command -v "$cmd" >/dev/null || fail "missing $cmd"; done
if command -v sha256sum >/dev/null 2>&1; then
  hash_file() { sha256sum "$1" | awk '{print $1}'; }
elif command -v shasum >/dev/null 2>&1; then
  hash_file() { shasum -a 256 "$1" | awk '{print $1}'; }
else
  fail 'no SHA-256 utility (sha256sum or shasum)'
fi
orm="$(realpath -e -- "$1")" || fail 'ORM root missing'
java_repo="$(realpath -e -- "$3")" || fail 'Java compiler root missing'
llvm_repo="$(realpath -e -- "$4")" || fail 'LLVM compiler root missing'
orm_head="$2"
llvm_head="$5"
evidence="$6"
[[ "$orm_head" =~ ^[a-f0-9]{40}$ && "$llvm_head" =~ ^[a-f0-9]{40}$ ]] || fail 'commit pins must be full lowercase 40-hex'
for dir in "$orm" "$java_repo" "$llvm_repo"; do
  [[ "$(git -C "$dir" rev-parse --show-toplevel)" == "$dir" ]] || fail "not a Git root: $dir"
done
[[ "$(git -C "$orm" rev-parse HEAD)" == "$orm_head" ]] || fail 'ORM HEAD mismatch'
[[ "$(git -C "$llvm_repo" rev-parse HEAD)" == "$llvm_head" ]] || fail 'LLVM HEAD mismatch'
[[ -z "$(git -C "$llvm_repo" status --porcelain --untracked-files=no)" ]] || fail 'dirty LLVM tracked source'
[[ -f "$orm/SOURCE_REF" ]] || fail 'Java SOURCE_REF not present'
java_head="$(tr -d '\r\n' < "$orm/SOURCE_REF")"
[[ "$java_head" =~ ^[a-f0-9]{40}$ ]] || fail 'invalid Java SOURCE_REF'
[[ "$(git -C "$java_repo" rev-parse HEAD)" == "$java_head" ]] || fail 'Java compiler HEAD mismatches SOURCE_REF'
[[ -x "$orm/scripts/ores" ]] || fail 'Java launcher missing'
[[ -x "$llvm_repo/scripts/compile-aot.sh" ]] || fail 'LLVM AOT compiler launcher missing'
llvmc="${ORESLANG_LLVMC:-$llvm_repo/build/oreslang-llvmc}"
[[ -x "$llvmc" && -f "$llvmc" && ! -L "$llvmc" ]] || fail 'LLVM compiler executable missing or symlink'
[[ -n "${ORES_JAVA:-}" ]] || fail 'set ORES_JAVA explicitly to reviewed JDK 25 binary'
[[ -x "$ORES_JAVA" && ! -L "$ORES_JAVA" ]] || fail 'JDK 25 binary missing or symlink'
"$ORES_JAVA" -version 2>&1 | head -n 1 | grep -Eq 'version "25([."]|$)' || fail 'JDK 25 required'
[[ ! -e "$evidence" ]] || fail 'evidence destination must not exist'
mkdir -m 0700 -- "$evidence" || fail 'cannot create evidence directory'
evidence="$(realpath -e -- "$evidence")"
root="$(cd "$(dirname "$0")/.." && pwd -P)"
{
  printf 'orm_commit=%s\n' "$orm_head"
  printf 'orm_tree=%s\n' "$(git -C "$orm" rev-parse 'HEAD^{tree}')"
  printf 'java_commit=%s\n' "$java_head"
  printf 'llvm_commit=%s\n' "$llvm_head"
  printf 'llvm_compiler_sha256=%s\n' "$(hash_file "$llvmc")"
  printf 'java_binary_sha256=%s\n' "$(hash_file "$ORES_JAVA")"
} > "$evidence/provenance.txt"
printf -v java_script '%q' "$orm/scripts/ores"
printf -v llvm_script '%q' "$llvm_repo/scripts/compile-aot.sh"
export ORESLANG_SOURCE_DIR="$java_repo" ORESLANG_LLVMC="$llvmc"
export ORES_PARITY_JAVA_COMPILE="$java_script --check {source}"
export ORES_PARITY_JAVA_RUN="$java_script {source}"
export ORES_PARITY_LLVM_COMPILE="$llvm_script {source} {artifact}"
export ORES_PARITY_LLVM_RUN="{artifact}"
failures=0
for backend in java llvm; do
  # Attempt each independent backend even if the other fails; retain both
  # raw result reports, never hide either failure or synthesize a success.
  if cargo run --locked --manifest-path "$root/tools/parity-runner/Cargo.toml" -- \
    --orm-root "$orm" --orm-head "$orm_head" --backend "$backend" \
    --case-id orm-schema-plans --case-id orm-schema-adversarial \
    --case-id orm-schema-invariants --case-id orm-sql-plans \
    --execute --allow-unvalidated --output "$evidence/$backend.json"; then
    printf '%s\n' "$backend candidate execution returned 0 (review receipts)"
  else
    failures=$((failures + 1))
    printf '%s\n' "$backend native guest attempt FAILED; inspect raw report" >&2
  fi
done
[[ "$failures" -eq 0 ]] || fail "$failures independent native backend lanes failed"
printf '%s\n' 'ORM_JAVA_LLVM_EXECUTED_CANDIDATES: inspect receipts for independent validation'
