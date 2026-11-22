#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(1, 16)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, max(2, n // 2)) for _ in range(n)])
