"""ioi19p2 generator: `n m a b`, then the m edges of a connected simple graph. Bushy trees
with extra edges that mostly lead to vertex 0 make some cases impossible to split."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 9
n = 100_000 if big else r.randint(3, max(3, size))
cap = n * (n - 1) // 2
m = 200_000 if big else r.randint(n - 1, min(cap, 2 * n))
star = not big and r.random() < 0.3  # small branches around vertex 0: often no split
if star:
    m = r.randint(n - 1, min(cap, n + 2))
edges = set()
for v in range(1, n):
    if star:
        u = 0 if r.random() < 0.8 else r.randrange(v)
    else:
        u = r.randrange(v) // 3 if r.random() < 0.7 else r.randrange(v)
    edges.add((u, v))
while len(edges) < m:
    if not star and r.random() < 0.6:
        u, v = 0, r.randrange(1, n)
    else:
        u, v = sorted(r.sample(range(n), 2))
    edges.add((u, v))
edges = list(edges)
r.shuffle(edges)
if star and n >= 6:
    a = r.randint(2, max(2, n // 3))
    b = r.randint(2, max(2, n - 1 - a - 1))
else:
    a = r.randint(1, n - 2)
    b = r.randint(1, n - 1 - a)
print(n, m, a, b)
for u, v in edges:
    print(u, v)
