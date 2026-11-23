#!/usr/bin/env python3
# checker.py input output [answer]: the reference's YES/NO; on YES, n numbers in 1..=1e9 whose
# median over every query's i, j, k is its x.
import sys
inp = list(map(int, open(sys.argv[1]).read().split()))
out = open(sys.argv[2]).read().split()
want = open(sys.argv[3]).read().split()[0] if len(sys.argv) > 3 else None
n, m = inp[0], inp[1]
if not out or out[0] not in ("YES", "NO"): sys.exit("no YES/NO")
if want and out[0] != want: sys.exit(f"said {out[0]}, reference {want}")
if out[0] == "NO":
    if len(out) != 1: sys.exit("output after NO")
    print("ok"); sys.exit()
if len(out) != 1 + n: sys.exit(f"{len(out) - 1} numbers for n = {n}")
a = [0] + [int(t) for t in out[1:]]
if not all(1 <= v <= 10**9 for v in a[1:]): sys.exit("number outside 1..=1e9")
for q in range(m):
    i, j, k, x = inp[2 + 4 * q: 6 + 4 * q]
    if sorted([a[i], a[j], a[k]])[1] != x: sys.exit(f"query {q + 1}: median is not {x}")
print("ok")
