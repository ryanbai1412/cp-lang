#!/usr/bin/env bash
# usage: loop.sh NAME [cases]
# NAME_w is the workaround version of NAME and shares its files
# runs cpl check, cpl test and cpl stress (against ../NAME_ref.cpp on ../NAME_gen.py
# inputs) on NAME.cp, then verify.sh: the codeforces build compiled with g++
set -u
name=$1; cases=${2:-200}; base=${name%_w}
dir=$(cd "$(dirname "$0")" && pwd); up=$(dirname "$dir"); cd "$dir"
[ -e "$name.tests" ] || cp -r "$up/$base.tests" "$name.tests"
extra=()
[ -e "$up/${base}_check.py" ] && extra=(--checker "$up/${base}_check.py")
cpl check "$name.cp" || exit 1
cpl test "$name" || exit 1
cpl stress "$name.cp" --brute "$up/${base}_ref.cpp" --gen "$up/${base}_gen.py" \
  --cases "$cases" --size 20 "${extra[@]}" || exit 1
"$dir/verify.sh" "$name" "$cases" 50
