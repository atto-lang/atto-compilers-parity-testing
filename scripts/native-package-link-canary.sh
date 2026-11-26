#!/usr/bin/env bash
# Linux host-linker proof, not an Atto compiler test. Static means package archive,
# not necessarily static libc or other system libraries.
set -euo pipefail
CC=${CC:-clang}
OPT=${OPT:-0}
case "$OPT" in 0|2) ;; *) exit 64;; esac
w=$(mktemp -d); trap 'rm -rf "$w"' EXIT
cd "$w"
cat > main.c <<'C'
#include <stdio.h>
extern int cart(void); extern int tax(void);
int main(void) { printf("%d\n", cart()+tax()); return 0; }
C
printf 'int cart(void){return 40;}\n' > cart.c
printf 'int tax(void){return 2;}\n' > tax.c
"$CC" "-O$OPT" -fPIC -c main.c -o main.o
"$CC" "-O$OPT" -fPIC -c cart.c -o cart.o
"$CC" "-O$OPT" -fPIC -c tax.c -o tax.o
ar rcs libcart.a cart.o; ar rcs libtax.a tax.o
"$CC" -shared -Wl,-soname,libcart.so cart.o -o libcart.so
"$CC" -shared -Wl,-soname,libtax.so tax.o -o libtax.so
"$CC" main.o libcart.a libtax.a -o static
"$CC" main.o -L. -lcart -ltax -Wl,-rpath,'$ORIGIN' -o dynamic
"$CC" main.o libtax.a -L. -lcart -Wl,-rpath,'$ORIGIN' -o hybrid
for mode in static dynamic hybrid; do [[ $(./"$mode") == 42 ]]; echo "PASS $mode=42"; done
if readelf -d static | grep -Eq 'NEEDED.*lib(cart|tax)\.so'; then exit 1; fi
readelf -d dynamic | grep -q 'NEEDED.*libcart.so'
readelf -d dynamic | grep -q 'NEEDED.*libtax.so'
readelf -d hybrid | grep -q 'NEEDED.*libcart.so'
if readelf -d hybrid | grep -q 'NEEDED.*libtax.so'; then exit 1; fi
echo 'PASS package dependency metadata matches all three modes'
sha256sum main.o tax.o dynamic hybrid > unchanged.sha
printf 'int cart(void){return 41;}\n' > cart.c
"$CC" "-O$OPT" -fPIC -c cart.c -o cart.o
ar rcs libcart.a cart.o
"$CC" -shared -Wl,-soname,libcart.so cart.o -o libcart.new.so
mv libcart.new.so libcart.so
sha256sum -c unchanged.sha
[[ $(./dynamic) == 43 && $(./hybrid) == 43 && $(./static) == 42 ]]
echo 'PASS dynamic/hybrid changed behavior on new process without relinking caller; static unchanged'
"$CC" main.o libcart.a libtax.a -o static.new
mv static.new static
[[ $(./static) == 43 ]]; sha256sum -c unchanged.sha
echo 'PASS static relink reuses caller and unrelated package object'
# Failed link must not overwrite previous good executable.
sha256sum static > prior.sha
if "$CC" main.o libtax.a -o invalid.new 2>link.err; then echo 'unexpected link success'; exit 1; fi
sha256sum -c prior.sha
[[ $(./static) == 43 ]]
echo 'PASS unresolved symbol rejected, prior executable preserved'
# Dynamic missing required package must fail to launch (no silent fallback).
mv libcart.so libcart.saved.so
if ./dynamic >missing.out 2>missing.err; then echo 'unexpected dynamic success'; exit 1; fi
echo 'PASS missing dynamic package rejected'
