#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(2, 15)
    k = rng.randint(2, n)
    print(n, k)
    print(*[rng.randint(0, n) for _ in range(n)])
