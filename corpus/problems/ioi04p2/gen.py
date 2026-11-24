#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n junctions with |X|, |Y| <= 1000.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 20000 over the full grid
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 20000 if MAX else r.randint(1, 8)
c = 1000 if MAX else r.choice([3, 10, 1000])
print(n)
for _ in range(n):
    print(r.randint(-c, c), r.randint(-c, c))
