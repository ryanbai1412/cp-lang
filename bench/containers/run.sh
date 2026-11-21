#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cpl="$root/target/debug/cpl"
[ -x "$cpl" ] || cargo build -q -p cpl-cli
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

benches=("$@")
if [ "${#benches[@]}" -eq 0 ]; then
    benches=(hash_insert hash_lookup hash_antihash bitset interval_map dsu monodeque flatmap sorted_range sorted_set ordered_map_set min_heap deque)
fi

ratios_file="$tmp/ratios.txt"
: > "$ratios_file"

for name in "${benches[@]}"; do
    "$cpl" build "$root/bench/containers/$name.cp" -o "$tmp/${name}_cp.cpp" >/dev/null
    g++ -std=c++17 -O2 "$tmp/${name}_cp.cpp" -o "$tmp/${name}_cp"
    g++ -std=c++17 -O2 "$root/bench/containers/$name.cpp" -o "$tmp/${name}_ref"
    python3 - "$name" "$tmp/${name}_cp" "$tmp/${name}_ref" "$ratios_file" <<'PY'
import subprocess, sys, time
name, cp_bin, ref_bin, rfile = sys.argv[1:5]
def best(b):
    out, ts = None, []
    for _ in range(3):
        t0 = time.perf_counter()
        r = subprocess.run([b], capture_output=True, text=True, check=True)
        ts.append((time.perf_counter() - t0) * 1000)
        out = r.stdout.strip()
    return min(ts), out
t_cp, o_cp = best(cp_bin)
t_ref, o_ref = best(ref_bin)
assert o_cp == o_ref, f"{name}: {o_cp} != {o_ref}"
ratio = t_cp / t_ref
with open(rfile, "a") as f:
    f.write(f"{ratio:.6f} {name}\n")
print(f"{name:16s} cp={t_cp:6.1f} ms  cpp={t_ref:6.1f} ms  ({ratio:.2f}x)")
PY
done

python3 - "$ratios_file" <<'PY'
import statistics, sys
rows = []
with open(sys.argv[1]) as f:
    for line in f:
        r, name = line.strip().split(None, 1)
        rows.append((float(r), name))
if rows:
    rs = [r for r, _ in rows]
    p50 = statistics.median(rs)
    mx, mx_name = max(rows)
    print(f"--- summary ({len(rows)} benches): p50={p50:.2f}x  max={mx:.2f}x ({mx_name})")
PY
