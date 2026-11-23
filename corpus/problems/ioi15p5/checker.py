#!/usr/bin/env python3
# checker.py input output [answer]: R is the fewest rounds (n - cycles of Ermek's first R swaps
# alone is at most R), followed by R index pairs that leave S sorted after round R - 1.
import sys
t = list(map(int, open(sys.argv[1]).read().split()))
n = t[0]; s = t[1:1 + n]; m = t[1 + n]; ermek = list(zip(t[2 + n::2], t[3 + n::2]))
def needed(r):
    p = s[:]
    for x, y in ermek[:r]: p[x], p[y] = p[y], p[x]
    seen = [False] * n; cycles = 0
    for i in range(n):
        if not seen[i]:
            cycles += 1; j = i
            while not seen[j]: seen[j] = True; j = p[j]
    return n - cycles
lo, hi = 0, m
while lo < hi:
    mid = (lo + hi) // 2
    if needed(mid) <= mid: hi = mid
    else: lo = mid + 1
out = list(map(int, open(sys.argv[2]).read().split()))
rounds = out[0]
if rounds != lo: sys.exit(f"R = {rounds}, fewest {lo}")
if len(out) != 1 + 2 * rounds: sys.exit(f"{(len(out) - 1) / 2} pairs for R = {rounds}")
a = s[:]
for i in range(rounds):
    x, y = ermek[i]; a[x], a[y] = a[y], a[x]
    p, q = out[1 + 2 * i], out[2 + 2 * i]
    if not (0 <= p < n and 0 <= q < n): sys.exit(f"round {i}: index out of range")
    a[p], a[q] = a[q], a[p]
if a != sorted(a): sys.exit("not sorted after R rounds")
print("ok")
