#!/usr/bin/env bash
# Time each std.math / std.mod / std.ntt / std.matrix benchmark against a matched C++17 implementation.
# Usage: bench/math/run.sh [name...]   (defaults to every *.cp here)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
cpl=${CPL:-$here/../../target/release/cpl}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
names=("$@")
[ ${#names[@]} -gt 0 ] || names=(modint ntt sieve matpow crt comb)

printf '%-10s %8s %8s %7s %8s\n' name cplang c++ ratio bytes
ratios=()
for n in "${names[@]}"; do
    "$cpl" build "$here/$n.cp" -o "$tmp/$n.gen.cpp" > /dev/null
    bytes=$(wc -c < "$tmp/$n.gen.cpp")
    g++ -std=c++17 -O2 -o "$tmp/$n.cpl" "$tmp/$n.gen.cpp"
    g++ -std=c++17 -O2 -o "$tmp/$n.ref" "$here/$n.cpp"
    line=$(python3 - "$n" "$tmp/$n.cpl" "$tmp/$n.ref" "$bytes" <<'PY'
import subprocess, sys, time
name, cp_bin, ref_bin, bytes_s = sys.argv[1:5]
def best(b):
    out, ts = None, []
    for _ in range(5):
        t0 = time.perf_counter()
        r = subprocess.run([b], capture_output=True, text=True, check=True)
        ts.append(time.perf_counter() - t0)
        out = r.stdout.strip()
    return min(ts), out
x, a = best(cp_bin)
y, b = best(ref_bin)
assert a == b, f"{name}: outputs differ: {a!r} vs {b!r}"
r = x / y
print(f"{name:<10s} {x:7.3f}s {y:7.3f}s {r:6.2f}x {int(bytes_s):7d}B|{r:.6f}")
PY
    )
    echo "${line%|*}"
    ratios+=("${line#*|}")
done
python3 - "${ratios[@]}" <<'PY'
import statistics, sys
rs = [float(x) for x in sys.argv[1:]]
print(f"p50={statistics.median(rs):.2f}x  max={max(rs):.2f}x")
PY
