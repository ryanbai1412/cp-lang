#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(3, 14)
    k = rng.randint(1, n)
    edges = []
    deg = [0] * (n + 1)
    for i in range(2, n + 1):
        p = rng.randint(1, i - 1)
        edges.append((p, i))
        deg[p] += 1
        deg[i] += 1
    non_leaves = [u for u in range(1, n + 1) if deg[u] >= 2]
    v = rng.choice(non_leaves)
    print(n, k, v)
    for u, w in edges:
        print(u, w)
