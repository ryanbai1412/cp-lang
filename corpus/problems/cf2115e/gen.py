#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
n = rng.randint(2, 8)
edges = set()
for i in range(2, n + 1):
    edges.add((rng.randint(1, i - 1), i))
for _ in range(rng.randint(0, n)):
    u = rng.randint(1, n - 1)
    v = rng.randint(u + 1, n)
    edges.add((u, v))
edges = sorted(edges)
print(n, len(edges))
for _ in range(n):
    print(rng.randint(1, 12), rng.randint(1, 50))
for u, v in edges:
    print(u, v)
q = rng.randint(1, 10)
print(q)
for _ in range(q):
    r = rng.randint(1, 250) if rng.random() < 0.5 else rng.randint(1, 100000)
    print(rng.randint(1, n), r)
