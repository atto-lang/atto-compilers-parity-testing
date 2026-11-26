#!/usr/bin/env bash
# Public compiler snapshots, no cross-org credentials, no Python.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
root=$PWD
lane=${1:?backend required}
manifest="$root/fixtures/inline-task-writeback-v1/manifest.json"
read -r head tree < <(node -e 'const m=require(process.argv[1]);const p=m.pins[process.argv[2]];if(!p)process.exit(2);console.log(p.join(" "))' "$manifest" "$lane")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
repo="$work/compiler"
git -c credential.helper= init -q "$repo"
git -C "$repo" remote add origin "https://github.com/atto-lang/atto-source.$lane.git"
GIT_TERMINAL_PROMPT=0 git -C "$repo" -c credential.helper= fetch --depth=1 origin "$head"
git -C "$repo" checkout -q --detach FETCH_HEAD
test "$(git -C "$repo" rev-parse HEAD)" = "$head"
test "$(git -C "$repo" rev-parse 'HEAD^{tree}')" = "$tree"
test -z "$(git -C "$repo" status --porcelain)"
printf 'backend=%s head=%s tree=%s platform=%s\n' "$lane" "$head" "$tree" "$(uname -sm)"
cd "$repo"
case "$lane" in
 llvm) cargo build --offline --release --bin atto-llvm; compiler="$repo/target/release/atto-llvm"; clang --version ;;
 rs) cargo build --locked --release --bin attoc; compiler="$repo/target/release/attoc"; node --version ;;
 js) compiler="$repo/bin/oreslang-jsc.mjs"; node --version ;;
 cranelift) cargo build --locked --release; compiler="$repo/target/release/orescranelift" ;;
 hdl) compiler="$repo/bin/oreslang-hdl.mjs"; node --version ;;
 *) echo "Unimplemented adapter: $lane" >&2; exit 2 ;;
esac
test -f "$compiler"
sha256sum "$compiler"
count=0
pass=0
while read -r name blob; do
  count=$((count+1))
  src="$root/fixtures/inline-task-writeback-v1/$name.atto"
  test "$(git hash-object "$src")" = "$blob"
  test "$(git -C "$root" rev-parse "HEAD:fixtures/inline-task-writeback-v1/$name.atto")" = "$blob"
  output="$work/$name"
  rc=0
  runtime=not_executed
  case "$lane" in
    llvm) timeout 120 "$compiler" "$src" -o "$output" >"$output.compile.log" 2>&1 || rc=$?
      if [[ "$rc" = 0 ]]; then
        test -x "$output"; sha256sum "$output"; runtime=0
        timeout 20 "$output" >"$output.run.log" 2>&1 || runtime=$?
        [[ "$runtime" = 42 ]] && pass=$((pass+1))
      fi ;;
    rs) timeout 120 "$compiler" "$src" -o "$output.wasm" >"$output.compile.log" 2>&1 || rc=$?
      if [[ "$rc" = 0 ]]; then
        sha256sum "$output.wasm"; runtime=0
        timeout 20 node --input-type=module -e 'import fs from "node:fs";import assert from "node:assert/strict";const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));assert.deepEqual(WebAssembly.Module.imports(m),[]);const i=new WebAssembly.Instance(m,{});assert.equal(i.exports.main(),42n);assert.equal(i.exports.panic_code.value,0);' "$output.wasm" >"$output.run.log" 2>&1 || runtime=$?
        [[ "$runtime" = 0 ]] && pass=$((pass+1))
      fi ;;
    js) timeout 120 node "$compiler" --atto-scalar-v1 "$src" "--out=$output.mjs" >"$output.compile.log" 2>&1 || rc=$?
      if [[ "$rc" = 0 ]]; then
        sha256sum "$output.mjs"; runtime=0
        timeout 20 node --input-type=module -e 'import assert from "node:assert/strict";const m=await import(process.argv[1]);assert.equal(m.main(),42n)' "$output.mjs" >"$output.run.log" 2>&1 || runtime=$?
        [[ "$runtime" = 0 ]] && pass=$((pass+1))
      fi ;;
    cranelift) timeout 120 "$compiler" check "$src" >"$output.compile.log" 2>&1 || rc=$?
      if [[ "$rc" = 0 ]]; then
        runtime=0
        timeout 20 "$compiler" run "$src" >"$output.run.log" 2>&1 || runtime=$?
        [[ "$runtime" = 0 && "$(cat "$output.run.log")" = 42 ]] && pass=$((pass+1))
      fi ;;
    hdl) timeout 120 node "$compiler" "$src" "$output.sv" >"$output.compile.log" 2>&1 || rc=$?
      # Negative admission only. Do not treat timeout/crash or arbitrary error as success.
      if [[ "$rc" = 1 && ! -e "$output.sv" ]] && grep -Eiq 'unsupported|not synthesizable|E_UNSUPPORTED' "$output.compile.log"; then pass=$((pass+1)); fi ;;
  esac
  test "$(git hash-object "$src")" = "$blob"
  printf 'case=%s blob=%s compile=%s runtime=%s\n' "$name" "$blob" "$rc" "$runtime"
  cat "$output.compile.log"
  if [[ -f "$output.run.log" ]]; then cat "$output.run.log"; fi
done < <(node -e 'for(const f of require(process.argv[1]).fixtures)console.log(f.name,f.blob)' "$manifest")
printf 'backend=%s outcomes=%d/%d scope=%s\n' "$lane" "$pass" "$count" "$([[ "$lane" = hdl ]] && echo negative-admission-only || echo guest)"
test "$count" = 4 && test "$pass" = 4
