#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n, L, B, then n sorted field coordinates in 1..=L.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 1e5, L = 1e9, B up to 2e15
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else r.randint(1, 8)
L = 10**9 if MAX else r.choice([5, 20, 10**9])
B = r.randint(0, 2 * 10**15) if MAX else r.choice([0, r.randint(0, 3 * L), r.randint(0, 2 * 10**15)])
print(n); print(L); print(B)
for x in sorted(r.randint(1, L) for _ in range(n)): print(x)
