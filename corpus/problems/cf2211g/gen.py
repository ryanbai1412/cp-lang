#!/usr/bin/env python3
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 6); print(t)
for _ in range(t):
    n = r.randint(1, max(1, S))
    hi = r.choice([3, 20, 10**6])
    print(n)
    print(*(r.randint(0, hi) for _ in range(n)))
