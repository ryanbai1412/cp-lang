#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 100000:
    # a long chain plus random forward edges: the longest path has n - 1 edges
    n, m = 100000, 100000
    edges = {(v, v + 1) for v in range(1, n)}
    while len(edges) < m:
        u = r.randint(1, n - 1)
        edges.add((u, r.randint(u + 1, n)))
else:
    n = r.randint(2, size + 2)
    m = r.randint(1, min(2 * size, n * (n - 1) // 2))
    edges = set()
    while len(edges) < m:
        u = r.randint(1, n - 1)
        edges.add((u, r.randint(u + 1, n)))
# relabel so topological order is not the label order
perm = list(range(1, n + 1))
r.shuffle(perm)
edges = [(perm[u - 1], perm[v - 1]) for u, v in edges]
r.shuffle(edges)
print(n, len(edges))
for u, v in edges:
    print(u, v)
