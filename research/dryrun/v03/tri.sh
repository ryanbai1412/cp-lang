#!/usr/bin/env bash
# usage: tri.sh NAME...: cpl check, cpl run on the first sample, cpl build --judge
# codeforces and g++ on the emitted C++; prints the first error line of each failing step
for name in "$@"; do
  base=${name%_w}
  w=${DRYRUN_TMP:-$HOME/.dryrun-tmp}/v03/$name; mkdir -p "$w"
  [ -e "$name.tests" ] || cp -r "../$base.tests" "$name.tests"
  in=$(ls "$name.tests"/*.in | head -1)
  c=$(cpl check "$name.cp" 2>&1) && c=ok || c=$(grep -m1 "^error" <<<"$c")
  r=$(timeout 20 cpl run "$name.cp" < "$in" 2>&1 >"$w/got") && { diff -bq "$w/got" "${in%.in}.out" >/dev/null && r=ok || r="wrong answer on $(basename "$in")"; } || r=$(grep -m1 "^error" <<<"$r")
  b=$(cpl build "$name.cp" --judge codeforces -o "$w/sol.cpp" 2>&1) && { b=$(g++ -O2 -std=c++17 -w "$w/sol.cpp" -o "$w/sol" 2>&1) && b=ok || b="g++: $(grep -m1 "error" <<<"$b" | sed 's|.*error: ||')"; } || b=$(grep -m1 "error" <<<"$b")
  printf '%s\n  check: %s\n  run:   %s\n  build: %s\n' "$name" "$c" "$r" "$b"
done
