#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; A B T, weak limits, small limits, then T toys `weight size`.
import random, sys
MAX = sys.argv[1] == "--max"  # A = B = 5e4, T = 1e6, values up to 2e9
r = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    a, b, t, top = 50000, 50000, 10**6, 2 * 10**9
else:
    a, b, t, top = r.randint(0, 4), r.randint(0, 4), r.randint(1, 10), r.choice([5, 20, 2 * 10**9])
    if a + b == 0: a = 1
print(a, b, t)
print(*[r.randint(1, top) for _ in range(a)])
print(*[r.randint(1, top) for _ in range(b)])
for _ in range(t): print(r.randint(1, top), r.randint(1, top))
