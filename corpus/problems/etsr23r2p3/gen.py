#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; t triples of values in 1..=1e9, often sharing a power of two.
import random, sys
MAX = sys.argv[1] == "--max"  # 1e5 triples near 1e9
r = random.Random(1 if MAX else int(sys.argv[1]))
t = 100000 if MAX else r.randint(1, 8)
print(t)
for _ in range(t):
    k = r.randint(0, 29)
    hi = 10**9 >> k
    if MAX:
        xs = [r.randint(1, 10**9) for _ in range(3)]
    elif r.random() < 0.6:
        xs = [r.randint(1, max(1, min(hi, 20))) << k for _ in range(3)]
    else:
        xs = [r.randint(1, 30) for _ in range(3)]
    print(*xs)
