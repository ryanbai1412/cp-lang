#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    k = rng.randint(1, 2 * n + 2)
    print(n, k)
