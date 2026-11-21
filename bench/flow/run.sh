#!/usr/bin/env bash
# Time each stdlib flow solver against a matched C++ implementation of the same algorithm.
# Usage: bench/flow/run.sh [name...]   (defaults to every *.cp here)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
cpl=${CPL:-$here/../../target/release/cpl}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
names=("$@")
[ ${#names[@]} -gt 0 ] || names=($(cd "$here" && ls *.cp | sed 's/\.cp$//'))
best() {
    python3 - "$1" <<'PY'
import subprocess, sys, time
times = []
for _ in range(5):
    t = time.perf_counter()
    subprocess.run([sys.argv[1]], stdout=subprocess.DEVNULL, check=True)
    times.append(time.perf_counter() - t)
print(f"{min(times):.3f}")
PY
}
printf '%-12s %8s %8s %6s\n' name cplang c++ ratio
for n in "${names[@]}"; do
    "$cpl" build "$here/$n.cp" -o "$tmp/$n.gen.cpp" > /dev/null
    g++ -std=c++17 -O2 -o "$tmp/$n.cpl" "$tmp/$n.gen.cpp"
    g++ -std=c++17 -O2 -o "$tmp/$n.ref" "$here/$n.cpp"
    a=$("$tmp/$n.cpl"); b=$("$tmp/$n.ref")
    [ "$a" = "$b" ] || { echo "$n: outputs differ: $a vs $b" >&2; exit 1; }
    x=$(best "$tmp/$n.cpl"); y=$(best "$tmp/$n.ref")
    printf '%-12s %8s %8s %6s\n' "$n" "$x" "$y" "$(python3 -c "print(f'{$x / $y:.2f}')")"
done
