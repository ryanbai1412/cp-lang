#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # one n = 1e6 test (sum n) of all 1e6: n * prefix reaches 1e18, no prefix falls below the
    # mean, and every cut is exact, so the block check walks the whole array
    n = 10**6
    print(1)
    print(n)
    print(" ".join(["1000000"] * n))
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 6); print(t)
for _ in range(t):
    n = r.randint(1, max(1, S))
    hi = r.choice([3, 20, 10**6])
    print(n)
    print(*(r.randint(0, hi) for _ in range(n)))
