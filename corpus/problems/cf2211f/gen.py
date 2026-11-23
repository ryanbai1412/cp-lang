#!/usr/bin/env python3
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 6); print(t)
for _ in range(t):
    hi = r.choice([3 + S, 50, 1000, 10**6])
    print(r.randint(3, hi), r.randint(3, hi))
