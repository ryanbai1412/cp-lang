#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(1, 14)
    a = [rng.randint(1, n) for _ in range(n)]
    if rng.random() < 0.3: a.sort()
    print(n)
    print(*a)
