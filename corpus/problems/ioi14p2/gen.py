#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n columns, k add/remove operations `op l r h`.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 2e6 columns, k = 5e5 operations
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 2 * 10**6 if MAX else r.randint(1, 8)
k = 5 * 10**5 if MAX else r.randint(1, 8)
H = 10**5 if MAX else r.choice([3, 10, 10**5])
print(n, k)
for _ in range(k):
    l = r.randrange(n)
    print(r.randint(1, 2), l, r.randint(l, n - 1), r.randint(0, H))
