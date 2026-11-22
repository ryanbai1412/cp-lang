#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, 6)
    m = rng.randint(1, 14)
    p = rng.randint(0, 100)
    print(n, m, p)
    print(*[rng.randint(1, 6) for _ in range(n)])
