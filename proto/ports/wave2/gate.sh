#!/bin/bash
# gate.sh NAME [CASES]: the four Wave 2 gates for one port, from anywhere:
# `cpl check`, `cpl build` compiled under ASan/UBSan, `cpl run`, and a
# differential run of both against the original accepted C++ on random inputs
# from gen/NAME.py. Needs `cargo build --release -p cpl-cli` first.
R=$(cd "$(dirname "$0")/../../.." && pwd); C=$R/target/release/cpl; P=$R/proto/ports
n=$1; N=${2:-100}; T=$(mktemp -d); trap 'rm -rf $T' EXIT
src=$P/wave2/$n.cp
orig=$P/${n}_orig.cpp; [ -f $orig ] || orig=$P/hillclimb_dp_graph/${n}_orig.cpp
$C check $src >/dev/null 2>$T/err || { echo "FAIL $n check"; head -8 $T/err; exit 1; }
$C build $src -o $T/out.cpp >/dev/null 2>$T/err || { echo "FAIL $n build"; head -8 $T/err; exit 1; }
g++ -std=c++17 -O1 -g -fsanitize=address,undefined -fno-sanitize-recover=all $T/out.cpp -o $T/san 2>$T/err || { echo "FAIL $n g++"; head -20 $T/err; exit 1; }
g++ -std=c++17 -O1 -w $orig -o $T/orig 2>$T/err || { echo "FAIL $n original g++"; head $T/err; exit 1; }
norm() { tr -s ' \n' '\n\n' < $1 | sed '/^$/d'; }
for s in $(seq 1 $N); do
  python3 $P/wave2/gen/$n.py $s $(( s % 8 + 1 )) > $T/in || { echo "FAIL $n gen"; exit 1; }
  timeout 10 $T/orig < $T/in > $T/o0 2>/dev/null || { echo "FAIL $n original crashed, seed $s"; exit 1; }
  timeout 20 $T/san < $T/in > $T/o1 2>$T/err || { echo "FAIL $n sanitizer run, seed $s"; head -8 $T/err; exit 1; }
  timeout 20 $C run $src < $T/in > $T/o2 2>$T/err || { echo "FAIL $n interpreter run, seed $s"; head -8 $T/err; exit 1; }
  for o in o1 o2; do
    diff -q <(norm $T/o0) <(norm $T/$o) >/dev/null || { echo "FAIL $n differs ($o), seed $s"; head -20 $T/in; exit 1; }
  done
done
echo "PASS $n ($N cases)"
