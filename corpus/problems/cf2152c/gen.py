#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
# statement: t <= 1e4, 1 <= n, q <= 250000, sums of n and of q <= 250000
MAX, MAX_T = 250_000, 10_000
if size < 1000:
    cases = [(rng.randint(1, max(4, size * 5)), rng.randint(1, max(3, size * 4))) for _ in range(rng.randint(1, 4))]
else:
    cases = [(MAX, MAX)] if rng.randrange(3) else [(MAX // MAX_T, MAX // MAX_T)] * MAX_T
print(len(cases))
for n, q in cases:
    print(n, q)
    alt = rng.random() < 0.4
    print(*[(i & 1) if alt and rng.random() < 0.95 else rng.randint(0, 1) for i in range(n)])
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n) if rng.random() < 0.5 else min(n, l + 3 * rng.randint(0, 20) + 2)
        print(l, r)
