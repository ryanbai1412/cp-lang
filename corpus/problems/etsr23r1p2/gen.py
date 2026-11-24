#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; `q n`, then q orders: `1` or `2 l r` with 1 <= l <= r <= n.
import random, sys
MAX = sys.argv[1] == "--max"  # 2e5 orders on 1..=1e9, short ranges so many stay held
r = random.Random(1 if MAX else int(sys.argv[1]))
q = 200000 if MAX else r.randint(1, 12)
n = 10**9 if MAX else r.choice([5, 20, 10**9])
span = 5000 if MAX else r.choice([1, 3, n])
print(q, n)
for _ in range(q):
    if r.random() < 0.2: print(1)
    else:
        l = r.randint(1, n)
        print(2, l, min(n, l + r.randint(0, span - 1)))
