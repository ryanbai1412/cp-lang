#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; `n lo hi` (0 <= lo <= hi), n operations `t x` (t in 1..=4,
# x >= 0), then `q` and q starting values >= 0.
import random, sys
MAX = sys.argv[1] == "--max"  # 1e5 operations, 1e5 starts; doubling keeps unclamped values at 0
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 100000 if MAX else r.randint(1, 8)
q = 100000 if MAX else r.randint(1, 8)
w = 10**9 if MAX else r.choice([5, 50, 10**9])
lo = r.randint(0, w // 2)
hi = r.randint(lo, w)
print(n, lo, hi)
for _ in range(n):
    t = r.randint(1, 4)
    print(t, r.randint(2, 3) if MAX and t == 3 else r.randint(0, w if t < 3 or r.random() < 0.1 else 3))
print(q)
print(*[r.randint(0, w) if r.random() < 0.9 else 0 for _ in range(q)])
