import random
import sys

# `seed size`: a connected simple graph on an odd n <= 2 size + 1 vertices with
# m <= 3 size edges (`--max`: n = 9999, m = 1e5).
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 0 if MAX else int(sys.argv[2])
n = 9999 if MAX else 2 * r.randint(0, size) + 1
most = n * (n - 1) // 2
m = 100_000 if MAX else min(most, r.randint(n - 1, n - 1 + 3 * size))
edges = set()
for v in range(2, n + 1):
    edges.add((r.randint(1, v - 1), v))
hub = r.randint(1, n)
while len(edges) < m:
    u = hub if r.random() < 0.3 else r.randint(1, n)
    v = r.randint(1, n)
    if u != v:
        edges.add((min(u, v), max(u, v)))
edges = list(edges)
r.shuffle(edges)
print(n, m)
for u, v in edges:
    print(u, v) if r.random() < 0.5 else print(v, u)
