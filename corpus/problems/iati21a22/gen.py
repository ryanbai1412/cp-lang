#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n points, small coordinates often colliding on a diagonal.
import random, sys
MAX = sys.argv[1] == "--max"  # 2e5 points over the full range
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 200000 if MAX else r.randint(1, 8)
c = 10**9 if MAX else r.choice([2, 5, 10**9])
print(n)
for _ in range(n):
    print(r.randint(-c, c), r.randint(-c, c))
