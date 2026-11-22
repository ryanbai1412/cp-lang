#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 50) for _ in range(n)])
    print(*[rng.randint(1, 50) for _ in range(n)])
