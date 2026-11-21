#!/usr/bin/env bash
# Every std.dp bench against its matched C++: best of 5 wall-clock runs,
# equal outputs asserted, then p50 and max of the ratios.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cpl="$root/target/release/cpl"
[ -x "$cpl" ] || cargo build -q --release -p cpl-cli
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
python3 "$root/bench/dp/gen.py" "$tmp"
# std.slope is cf2084g2's port at n = 5e5 against the original's slope treap.
python3 "$root/proto/ports/ecnerwala/gen/cf2084g2.py" 1 500000 > "$tmp/slope.in"
for name in frog3 dnc knuth mono monge aliens slope; do
    src="$root/bench/dp/$name.cp"; ref="$root/bench/dp/${name}_matched.cpp"
    if [ "$name" = slope ]; then src="$root/proto/ports/ecnerwala/cf2084g2.cp"; fi
    "$cpl" build "$src" -o "$tmp/${name}_cp.cpp" >/dev/null
    g++ -std=c++17 -O2 "$tmp/${name}_cp.cpp" -o "$tmp/${name}_cp"
    g++ -std=c++17 -O2 "$ref" -o "$tmp/${name}_ref" 2>/dev/null
    python3 - "$name" "$tmp" "$(wc -c < "$tmp/${name}_cp.cpp")" <<'PY'
import subprocess, sys, time
name, tmp, size = sys.argv[1:4]
def best(b):
    ts, out = [], None
    for _ in range(5):
        with open(f"{tmp}/{name}.in") as f:
            t0 = time.perf_counter()
            out = subprocess.run([b], stdin=f, capture_output=True, text=True, check=True).stdout
            ts.append(time.perf_counter() - t0)
    return min(ts), out
t_cp, o_cp = best(f"{tmp}/{name}_cp")
t_ref, o_ref = best(f"{tmp}/{name}_ref")
assert o_cp == o_ref, f"{name}: {o_cp!r} != {o_ref!r}"
print(f"{name:7s} cpp={t_ref:.3f} s  cp={t_cp:.3f} s  ratio={t_cp / t_ref:.2f}x  emitted={size} B")
with open(f"{tmp}/ratios", "a") as f:
    f.write(f"{t_cp / t_ref}\n")
PY
done
python3 - "$tmp/ratios" <<'PY'
import statistics, sys
r = sorted(float(x) for x in open(sys.argv[1]))
print(f"p50 {statistics.median(r):.2f}x (target <= 1.1x), max {r[-1]:.2f}x (target <= 1.5x)")
PY
