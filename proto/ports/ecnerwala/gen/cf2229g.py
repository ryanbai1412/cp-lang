#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    k = rng.randint(1, 40)
    x = rng.randint(1, n)
    print(n, k, x)
    print(*[rng.randint(1, 25) for _ in range(n)])
    print(*[rng.randint(0, 30) for _ in range(n - 1)])
