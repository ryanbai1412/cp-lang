#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(3, size * 4))
    k = rng.randint(1, n)
    q = rng.randint(1, max(2, size * 3))
    print(n, k, q)
    for _ in range(q):
        c = rng.choice([1, 2])
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        print(c, l, r)
