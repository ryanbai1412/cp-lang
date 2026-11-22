#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 14)
    edges = set()
    m = rng.randint(0, n)
    for _ in range(m):
        u = rng.randint(1, n - 1)
        v = rng.randint(u + 1, n)
        edges.add((u, v))
    print(n, len(edges))
    for u, v in sorted(edges):
        print(u, v)
