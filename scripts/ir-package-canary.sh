#!/usr/bin/env bash
# Source-free native LLVM IR package linking canary; not an Atto compiler test.
set -euo pipefail
opt=${OPT:-2}; mode=${LTO:-off}
case "$opt" in 0|2) ;; *) exit 64;; esac
flags=(); ldflags=()
case "$mode" in
  off) ;;
  full) flags=(-flto); ldflags=(-flto -fuse-ld=lld);;
  thin) flags=(-flto=thin); ldflags=(-flto=thin -fuse-ld=lld);;
  *) exit 64;;
esac
w=$(mktemp -d); trap 'rm -rf "$w"' EXIT
cd "$w"
cat > consumer.ll <<'IR'
declare i32 @cart()
declare i32 @tax()
define i32 @main() {
  %a = call i32 @cart()
  %b = call i32 @tax()
  %r = add i32 %a, %b
  ret i32 %r
}
IR
printf 'define i32 @cart() { ret i32 40 }\n' > cart.ll
printf 'define i32 @tax() { ret i32 2 }\n' > tax.ll
compile() { clang "-O$opt" "${flags[@]}" -fPIC -x ir -c "$1.ll" -o "$1.o"; }
run_exit() { local rc=0; "./$1" || rc=$?; [[ "$rc" == "$2" ]]; }
for unit in consumer cart tax; do compile "$unit"; done
llvm-ar rcs libcart.a cart.o
llvm-ar rcs libtax.a tax.o
clang "-O$opt" "${ldflags[@]}" consumer.o libcart.a libtax.a -o static
clang "-O$opt" "${ldflags[@]}" -shared -Wl,-soname,libcart.so cart.o -o libcart.so
clang "-O$opt" "${ldflags[@]}" consumer.o libtax.a -L. -lcart -Wl,-z,now -Wl,-rpath,'$ORIGIN' -o hybrid
run_exit static 42; run_exit hybrid 42
if readelf -dW static | grep -Eq 'NEEDED.*libcart.so'; then exit 1; fi
readelf -dW hybrid | grep -Eq 'NEEDED.*libcart.so'
sha256sum consumer.o tax.o libtax.a hybrid > reuse.sha
printf 'define i32 @cart() { ret i32 41 }\n' > cart.ll
compile cart; llvm-ar rcs libcart.a cart.o
clang "-O$opt" "${ldflags[@]}" -shared -Wl,-soname,libcart.so cart.o -o libcart.next.so
mv libcart.next.so libcart.so
run_exit static 42; run_exit hybrid 43
clang "-O$opt" "${ldflags[@]}" consumer.o libcart.a libtax.a -o static.next
mv static.next static
run_exit static 43; sha256sum -c reuse.sha
sha256sum static > prior.sha
printf 'define i32 @removed_cart() { ret i32 41 }\n' > cart.ll
compile cart
llvm-ar rcs libbroken.a cart.o
if clang "-O$opt" "${ldflags[@]}" consumer.o libbroken.a libtax.a -o rejected 2>link.err; then exit 1; fi
[[ ! -e rejected ]]; sha256sum -c prior.sha
clang "-O$opt" "${ldflags[@]}" -shared -Wl,-soname,libcart.so cart.o -o libcart.next.so
mv libcart.next.so libcart.so
if ./hybrid >bad.out 2>bad.err; then exit 1; fi
grep -q 'undefined symbol: cart' bad.err
run_exit static 43
printf 'PASS IR_PACKAGE_HOST_ONLY opt=%s lto=%s independent_objects=3 compatible_update=43 abi_removal_rejected=yes atto_guest=NOT_EXECUTED\n' "$opt" "$mode"
