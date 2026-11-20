#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 3)
print(T)
for _ in range(T):
    n = rng.randint(2, 7)
    k = rng.randint(1, n - 1)
    print(n, k)
    print(*[rng.randint(1, 30) for _ in range(n)])
