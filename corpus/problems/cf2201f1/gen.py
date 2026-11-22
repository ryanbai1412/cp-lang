#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 10)
    q = rng.randint(1, n)
    print(n, q)
    c0 = [0] * (n + 1)
    c1 = [0] * (n + 1)
    for _ in range(q):
        r0 = rng.choice([i for i in range(1, n + 1) if c0[i] < n - 1])
        r1 = rng.choice([i for i in range(1, n + 1) if c1[i] < n - 1])
        c0[r0] += 1; c1[r1] += 1
        print(r0, r1)
