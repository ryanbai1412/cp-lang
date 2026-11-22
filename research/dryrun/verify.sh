#!/usr/bin/env bash
# usage: verify.sh NAME [cases] [size]
# builds NAME.cp for codeforces, compiles the C++ with g++ -O2 -std=c++17,
# and compares it with NAME_ref.cpp on inputs from NAME_gen.py
set -u
name=$1; cases=${2:-200}; size=${3:-50}
dir=$(cd "$(dirname "$0")" && pwd); cd "$dir"
w=${DRYRUN_TMP:-$HOME/.dryrun-tmp}/$name; mkdir -p "$w"
cpl build "$name.cp" --judge codeforces -o "$w/sol.cpp" || exit 1
g++ -O2 -std=c++17 "$w/sol.cpp" -o "$w/sol" 2> "$w/sol.err" || { echo "g++ failed on emitted C++"; head -30 "$w/sol.err"; exit 1; }
g++ -O2 -std=c++17 "${name}_ref.cpp" -o "$w/ref" || exit 1
for f in "$name".tests/*.in; do
  [ -e "$f" ] || continue
  o=${f%.in}.out
  if [ -e "$o" ] && ! diff -bq <("$w/sol" < "$f") "$o" >/dev/null; then echo "sample $f differs"; fi
done
fail=0
for s in $(seq 1 "$cases"); do
  python3 "${name}_gen.py" "$s" "$size" > "$w/in"
  "$w/ref" < "$w/in" > "$w/exp"
  timeout 10 "$w/sol" < "$w/in" > "$w/got"; rc=$?
  if [ -e "${name}_check.py" ]; then python3 "${name}_check.py" "$w/in" "$w/got" "$w/exp"; ok=$?; else diff -bq "$w/exp" "$w/got" >/dev/null; ok=$?; fi
  if [ $rc -ne 0 ] || [ $ok -ne 0 ]; then
    echo "MISMATCH seed=$s rc=$rc"; head -c 600 "$w/in"; echo "--- exp"; head -c 300 "$w/exp"; echo "--- got"; head -c 300 "$w/got"; fail=1; break
  fi
done
[ $fail = 0 ] && echo "OK $name: $cases random cases match (size $size)"
