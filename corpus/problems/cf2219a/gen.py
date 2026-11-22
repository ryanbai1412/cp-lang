#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 10)
print(T)
for _ in range(T):
    if rng.random() < 0.5:
        n = rng.randint(1, 30)
        m = rng.randint(1, 30)
        q = rng.randint(0, n * m + min(n, m))
        p = 2 * n * m + n + m - 2 * q
        if p >= 0:
            print(p, q)
            continue
    print(rng.randint(0, 200), rng.randint(0, 200))
