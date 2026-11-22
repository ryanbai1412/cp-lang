#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cpl="${CPL:-$root/target/release/cpl}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

for name in scan scan_in_place rotate slice_sort slice_nth_element; do
    "$cpl" build "$root/bench/seq/$name.cp" -o "$tmp/${name}_cp.cpp" >/dev/null
    bytes=$(wc -c < "$tmp/${name}_cp.cpp")
    g++ -std=c++17 -O2 "$tmp/${name}_cp.cpp" -o "$tmp/${name}_cp"
    g++ -std=c++17 -O2 "$root/bench/seq/$name.cpp" -o "$tmp/${name}_ref"
    python3 - "$name" "$tmp/${name}_cp" "$tmp/${name}_ref" "$bytes" <<'PY'
import subprocess, sys, time
name, cp_bin, ref_bin, bytes_s = sys.argv[1:5]
def best(b):
    out, ts = None, []
    for _ in range(5):
        t0 = time.perf_counter()
        r = subprocess.run([b], capture_output=True, text=True, check=True)
        ts.append((time.perf_counter() - t0) * 1000)
        out = r.stdout.strip()
    return min(ts), out
t_cp, o_cp = best(cp_bin)
t_ref, o_ref = best(ref_bin)
assert o_cp == o_ref, f"{name}: {o_cp} != {o_ref}"
print(f"{name:18s} cp={t_cp:6.1f} ms  cpp={t_ref:6.1f} ms  ratio={t_cp/t_ref:.2f}x  emitted={bytes_s} B")
PY
done
