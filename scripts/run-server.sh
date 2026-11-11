#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$root/scripts/lib/guards.sh"
# TEST-ONLY overlay: source dependency repositories were copied as exact trees
# to this private GitHub repo. Never fetch private submodules with GITHUB_TOKEN.
export ORESLANG_SOURCE_DIR="$root/.cache/oreslang-source.java"
if [[ -z "${ORES_JAVA:-}" && -n "${JAVA_HOME:-}" ]]; then
  export ORES_JAVA="$JAVA_HOME/bin/java"
fi
[[ "$(ores_read_exact_source_ref "$root")" == "4b0ff7dd4063df2906dc2fed0f6b6348b69807c6" ]] ||
  { echo 'Unexpected compiler pin' >&2; exit 65; }
[[ "$(git -C "$root" rev-parse HEAD:.cache/oreslang-source.java)" == "bed52a435ebce6b2100bface266ae1a38a09e566" ]] ||
  { echo 'Compiler tree identity mismatch' >&2; exit 65; }
[[ "$(git -C "$root" rev-parse HEAD:dependencies/spin/dependencies/http-routing)" == "97d55d96901887a60f3cdac60f8af4f14b05e4c7" ]] ||
  { echo 'HTTP routing tree mismatch' >&2; exit 65; }
[[ "$(git -C "$root" rev-parse HEAD:dependencies/otel)" == "60c6f66a680b315742ec9a3cffe7ec031fd85515" ]] ||
  { echo 'OTel tree mismatch' >&2; exit 65; }
[[ -s "$ORESLANG_SOURCE_DIR/target/classpath.txt" ]] ||
  { echo 'Pinned Java compiler has not been built' >&2; exit 69; }

port="${PORT:-3000}"
if [[ ! "$port" =~ ^[0-9]{1,5}$ ]] || (( 10#$port < 1 || 10#$port > 65535 )); then
  echo 'PORT must be a decimal integer in 1..65535.' >&2
  exit 64
fi

data_input="${DATA_DIR:-$root/data}"
[[ -n "$data_input" ]] || { echo 'DATA_DIR cannot be empty.' >&2; exit 64; }
mkdir -p -- "$data_input"
data_dir="$(cd -- "$data_input" && pwd -P)"
if [[ "$data_dir" == "/" ]]; then
  echo 'Refusing to grant demo filesystem capabilities to the filesystem root.' >&2
  exit 64
fi
if [[ "$data_dir" == *","* ]]; then
  echo 'DATA_DIR cannot contain a comma because runtime capability lists are comma-separated.' >&2
  exit 64
fi

python3 "$root/scripts/generate-routes.py" --port "$port" --data-dir "$data_dir"
cd "$root"
exec bash "$root/dependencies/spin/scripts/compiler.sh" --platform=server \
  --allow-net="127.0.0.1:$port" --allow-read="$data_dir" --allow-write="$data_dir" "$@" "$root/src/main.ores"
