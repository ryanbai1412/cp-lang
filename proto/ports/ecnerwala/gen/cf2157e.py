#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(3, size * 5))
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 2 * n) for _ in range(n)])
