#!/usr/bin/env bash
# usage: run.sh [FILE...]: the first error of cpl check, cpl run and the g++ build of each repro
cd "$(dirname "$0")"; w=${DRYRUN_TMP:-$HOME/.dryrun-tmp}/repro-v03; mkdir -p "$w"
for f in "${@:-*.cp}"; do
  [ "$#" -eq 0 ] && set -- *.cp
done
for f in "$@"; do
  c=$(cpl check "$f" 2>&1) && c=ok || c=$(grep -m1 "^error" <<<"$c")
  r=$(timeout 10 cpl run "$f" </dev/null 2>&1) && r="ok: $(tr '\n' ' ' <<<"$r")" || r=$(grep -m1 "^error" <<<"$r")
  b=$(cpl build "$f" --judge codeforces -o "$w/x.cpp" 2>&1) && { b=$(g++ -O2 -std=c++17 -w "$w/x.cpp" -o "$w/x" 2>&1) && b=ok || b="g++: $(grep -m1 "error" <<<"$b" | sed 's|.*error: ||' | cut -c1-120)"; } || b=$(grep -m1 "error" <<<"$b")
  printf '%s\n  check: %s\n  run:   %s\n  build: %s\n' "$f" "$c" "$r" "$b"
done
