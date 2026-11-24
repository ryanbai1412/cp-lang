#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n = sum q = 2e5, values up to 1e9
    rng = random.Random(1)
    n = q = 200_000
    print(1)
    print(n, q)
    print(" ".join(str(rng.randint(1, 10**9)) for _ in range(n)))
    print("\n".join(f"{rng.randint(1, n)} {rng.randint(1, 10**9)}" for _ in range(q)))
    sys.exit()

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
