#!/usr/bin/env python3
# apio09p2: n requests [s, e], 1 <= n <= 2e5, 1 <= s <= e <= 1e9.
import random, sys
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2]) if len(sys.argv) > 2 else 12
n = r.randint(1, S)
hi = r.choice([3, 10, 3 * n, 10**9])
lim = r.choice([1, 3, max(1, hi // 4), hi])
print(n)
for _ in range(n):
    s = r.randint(1, hi)
    e = min(hi, s + r.randint(0, lim - 1))
    print(s, e)
