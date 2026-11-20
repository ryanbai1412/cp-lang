#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(1, 1000)
    m = rng.randint(0, 3000)
    k = rng.randint(1, n)
    print(n, m, k)
