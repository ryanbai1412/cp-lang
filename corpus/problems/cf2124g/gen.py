#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(2, 16)
    print(n)
    print(*[rng.randint(1, 20) for _ in range(n)])
