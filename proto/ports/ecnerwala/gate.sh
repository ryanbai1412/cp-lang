#!/bin/bash
# gate.sh NAME [CASES]: gates for one ecnerwala port, on the Rust `cpl`:
# `cpl check`, `cpl build` compiled under ASan/UBSan, `cpl run`, then both
# compared with the original C++ on the committed samples and on CASES random
# inputs from gen/NAME.py (when there is one).
# INTERP_TIMEOUT (seconds, default 20) bounds each `cpl run`; a port that needs
# more than the default is listed with its reason in the README.
# Needs `cargo build --release -p cpl-cli` first, and `zig` for the C++23 originals.
D=$(cd "$(dirname "$0")" && pwd); R=$(cd "$D/../../.." && pwd); C=$R/target/release/cpl
n=$1; N=${2:-100}; T=$(mktemp -d); trap 'rm -rf $T' EXIT
src=$D/$n.cp; orig=$D/${n}_orig.cpp
$C check $src >/dev/null 2>$T/err || { echo "FAIL $n check"; head -8 $T/err; exit 1; }
$C build $src -o $T/out.cpp >/dev/null 2>$T/err || { echo "FAIL $n build"; head -8 $T/err; exit 1; }
g++ -std=c++17 -O1 -g -fsanitize=address,undefined -fno-sanitize-recover=all $T/out.cpp -o $T/san 2>$T/err || { echo "FAIL $n g++"; head -20 $T/err; exit 1; }
# Some originals are C++23 (deducing `this`), which g++ 11 rejects: fall back to
# `zig c++` (clang 20, libc++), giving it the <bits/stdc++.h> that libc++ lacks.
if ! g++ -std=c++20 -O1 -w $orig -o $T/orig 2>$T/err; then
  mkdir -p $T/inc/bits
  for h in algorithm array bitset cassert chrono climits cmath cstdint cstdio cstdlib cstring deque functional iomanip iostream map numeric optional queue random set sstream stack string tuple unordered_map unordered_set utility vector; do
    echo "#include <$h>"; done > $T/inc/bits/stdc++.h
  zig c++ -std=c++23 -O1 -w -I$T/inc $orig -o $T/orig 2>$T/err || { echo "FAIL $n original does not compile"; head $T/err; exit 1; }
fi
norm() { tr -s ' \n' '\n\n' < $1 | sed '/^$/d'; }
one() { # one INPUT LABEL
  timeout 10 $T/orig < $1 > $T/o0 2>/dev/null || { echo "FAIL $n original crashed, $2"; exit 1; }
  timeout 20 $T/san < $1 > $T/o1 2>$T/err || { echo "FAIL $n sanitizer run, $2"; head -8 $T/err; exit 1; }
  timeout ${INTERP_TIMEOUT:-20} $C run $src < $1 > $T/o2 2>$T/err || { echo "FAIL $n interpreter run, $2"; head -8 $T/err; exit 1; }
  for o in o1 o2; do
    diff -q <(norm $T/o0) <(norm $T/$o) >/dev/null || { echo "FAIL $n differs ($o), $2"; head -20 $1; exit 1; }
  done
}
S=0
for f in $D/samples/$n.*.in; do [ -f $f ] || continue; one $f "sample $(basename $f)"; S=$((S+1)); done
G=0
if [ -f $D/gen/$n.py ]; then
  for s in $(seq 1 $N); do
    python3 $D/gen/$n.py $s $(( s % 8 + 1 )) > $T/in || { echo "FAIL $n gen"; exit 1; }
    one $T/in "seed $s"; G=$((G+1))
  done
fi
echo "PASS $n ($S samples, $G random cases)"
