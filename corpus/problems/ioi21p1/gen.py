#!/usr/bin/env python3
"""gen.py seed size | --max: IOI 2021 Distributing Candies. n, q <= 2e5, capacities <= 1e9,
days l r v with 0 <= l <= r < n and 0 < |v| <= 1e9. --max: n = q = 2e5, wide ranges and
alternating large fills and drains, so the boxes keep hitting both walls."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 200_000 if MAX else int(sys.argv[2])
n = S if MAX else r.randint(1, max(1, S))
q = S if MAX else r.randint(1, max(1, S))
C = 10 ** 9 if MAX or r.random() < 0.5 else 10
print(n)
print(*[r.randint(1, C) for _ in range(n)])
print(q)
ops = []
for d in range(q):
    l = r.randint(0, n - 1) if not MAX else r.randint(0, n // 10)
    rr = r.randint(l, n - 1) if not MAX else r.randint(n - 1 - n // 10, n - 1)
    v = r.randint(1, C) * (1 if d % 2 == 0 else -1) if MAX else r.choice([-1, 1]) * r.randint(1, C)
    ops.append(f"{l} {rr} {v}")
print("\n".join(ops))
