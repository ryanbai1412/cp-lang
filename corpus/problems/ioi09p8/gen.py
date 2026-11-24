#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; `n U D S`, then n fairs `day location profit`, the fair
# locations distinct and different from home S, D <= U.
import random, sys
MAX = sys.argv[1] == "--max"  # 5e5 fairs over 1000 days on the whole river
r = random.Random(1 if MAX else int(sys.argv[1]))
top = 500001 if MAX else r.choice([10, 30, 500001])
n = 500000 if MAX else r.randint(1, min(8, top - 1))
d = 10 if MAX else r.randint(1, 10)
u = 10 if MAX else r.randint(d, 10)
days = 1000 if MAX else r.randint(1, 4)
spots = r.sample(range(1, top + 1), n + 1)
print(n, u, d, spots[0])
for at in spots[1:]: print(r.randint(1, days), at, r.randint(1, 4000))
