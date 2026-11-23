#!/usr/bin/env python3
"""gen.py SEED [N_MAX]: a case for interactor.py, a simple connected graph (2 <= n, 1 <= m <= 4e5).

Odd seeds even out each component's count of vertex 1's neighbours (outside vertex 1) where
an extra edge to 1 can, so most of them are wins to play out; even seeds leave the graph raw,
mostly a resign. The last line is Maria's seed (the case seed; she plays at random)."""
import random, sys

seed = int(sys.argv[1])
n_max = int(sys.argv[2]) if len(sys.argv) > 2 else 60
rng = random.Random(seed)
n = rng.randint(2, max(2, n_max))
edges = set()
order = list(range(1, n + 1))
rng.shuffle(order)
for i in range(1, n):
    a, b = order[i], order[rng.randrange(i)]
    edges.add((min(a, b), max(a, b)))
extra = min(rng.choice([0, n // 4, n, 2 * n]), max(0, 400000 - (n - 1) - n // 3 - n))
for _ in range(extra):
    a, b = rng.sample(range(1, n + 1), 2) if n > 2 else (1, 2)
    edges.add((min(a, b), max(a, b)))
for _ in range(rng.randint(0, n // 3)):
    edges.add((1, rng.randint(2, n))) if n > 2 else None
if seed % 2 == 1:
    up = list(range(n + 1))
    def find(x):
        while up[x] != x:
            up[x] = up[up[x]]
            x = up[x]
        return x
    for a, b in edges:
        if a != 1:
            up[find(a)] = find(b)
    friends = {b for a, b in edges if a == 1}
    for r in {find(f) for f in friends}:
        comp = [v for v in range(2, n + 1) if find(v) == r]
        if sum(v in friends for v in comp) % 2 == 1:
            free = [v for v in comp if v not in friends]
            if free:
                edges.add((1, rng.choice(free)))
edges = list(edges)
rng.shuffle(edges)
edges = [(b, a) if rng.random() < 0.5 else (a, b) for a, b in edges]
print(n, len(edges))
for a, b in edges:
    print(a, b)
print(seed)
