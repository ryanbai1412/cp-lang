#!/usr/bin/env python3
import random
import sys

seed = int(sys.argv[1])
size = int(sys.argv[2])
rng = random.Random(seed)

if size >= 10000:
    t = 1
    n = size
    q = size
    print(t)
    print(n, q)
    print(" ".join(str(rng.randint(1, 10**9)) for _ in range(n)))
    for _ in range(q):
        print(rng.randint(1, n), rng.randint(1, 10**9))
else:
    t = rng.randint(1, 4)
    print(t)
    for _ in range(t):
        n = rng.randint(1, max(1, size))
        q = rng.randint(1, max(1, size))
        lim = 20 if seed % 2 == 0 else 10**9
        print(n, q)
        print(" ".join(str(rng.randint(1, lim)) for _ in range(n)))
        for _ in range(q):
            print(rng.randint(1, n), rng.randint(1, lim))
