#!/usr/bin/env python3
"""gen.py seed size | --max: CSP-S 2019 Partition. n <= 4e7, a_i in 1..1e9, either listed
(`n 0`) or from the b-recurrence generator (`n 1`). --max: n = 4e7 by generator with m = 1e5
parts, half of them narrow so the sums stay close to non-decreasing."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 40_000_000 if MAX else r.randint(2, max(2, min(40_000_000, int(sys.argv[2]))))
kind = 1 if MAX or r.random() < 0.4 else 0
top = r.choice([3, 10, 1_000_000_000])
print(n, kind)
if kind == 0:
    print(*(r.randint(1, top) for _ in range(n)))
else:
    m = 100_000 if MAX else r.randint(1, min(n, 5))
    cuts = sorted(r.sample(range(1, n), m - 1)) + [n]
    print(*(r.randrange(1 << 30) for _ in range(5)), m)
    for p in cuts:
        l = r.randint(1, top)
        hi = top if MAX and r.random() < 0.5 else min(1_000_000_000, l + r.randint(0, 3))
        print(p, l, max(l, hi))
