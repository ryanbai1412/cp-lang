#!/usr/bin/env python3
# checker.py input output [answer]: the fewest moves M (the accepted solution's closed form),
# then M distinct sizes 1..n-1, signed, that keep the piece on 1..n and leave no move that fits.
import sys
n, x = map(int, open(sys.argv[1]).read().split())
out = list(map(int, open(sys.argv[2]).read().split()))
least = 0 if n == 1 else n // 2 + (n % 2 == 1 and ((n - 3) // 2 % 4 in (0, 3)) != (x % 2 == 1))
m = out[0]
if m != least: sys.exit(f"M = {m}, fewest {least}")
if len(out) != 1 + m: sys.exit(f"{len(out) - 1} moves for M = {m}")
used = set(); at = x
for d in out[1:]:
    if not 1 <= abs(d) < n or abs(d) in used: sys.exit(f"size {d} out of range or reused")
    used.add(abs(d)); at += d
    if not 1 <= at <= n: sys.exit(f"left the board at {at}")
for s in range(1, n):
    if s not in used and (at + s <= n or at - s >= 1): sys.exit(f"size {s} still fits at {at}")
print("ok")
