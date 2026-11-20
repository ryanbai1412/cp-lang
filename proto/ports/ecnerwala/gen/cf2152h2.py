#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(2, max(3, size * 4))
    print(n)
    for v in range(2, n + 1):
        u = rng.randint(1, v - 1)
        w = rng.randint(1, 20)
        print(u, v, w)
    q = rng.randint(1, max(2, size * 3))
    print(q)
    for _ in range(q):
        print(rng.randint(1, 40))
