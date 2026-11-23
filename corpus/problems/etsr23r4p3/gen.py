#!/usr/bin/env python3
"""ZJOI 2016 Small Stars: n <= 17, a simple graph with m <= n(n-1)/2 edges, a tree on 1..n."""
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 1000:
    n = 17
    mode = int(sys.argv[1]) % 5
    p = [1.0, 1.0, 1.0, 0.9, 0.7][mode]
    shape = ["path", "star", "rand", "rand", "path"][mode]
else:
    n = r.randint(1, 8)
    p = r.choice([0.3, 0.6, 1.0])
    shape = r.choice(["path", "star", "rand"])
edges = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1) if r.random() < p]
r.shuffle(edges)
perm = list(range(1, n + 1))
r.shuffle(perm)
print(n, len(edges))
for u, v in edges:
    print(*((u, v) if r.random() < 0.5 else (v, u)))
for v in range(2, n + 1):
    u = {"path": v - 1, "star": 1, "rand": r.randint(1, v - 1)}[shape]
    print(perm[u - 1], perm[v - 1])
