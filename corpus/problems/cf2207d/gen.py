#!/usr/bin/env python3
# usage: gen.py seed size; size caps the sum of n (the statement allows 5e5)
import random, sys
rng = random.Random(int(sys.argv[1]))
budget = max(3, min(int(sys.argv[2]) if len(sys.argv) > 2 else 14, 500_000))
sizes = []
while budget >= 3 and len(sizes) < 10_000:
    n = budget if rng.random() < 0.3 else rng.randint(3, budget)
    sizes.append(n)
    budget -= n
print(len(sizes))
for n in sizes:
    shape = rng.choice(["random", "path", "star", "caterpillar", "broom", "deep"])
    par = [0] * (n + 1)
    for i in range(2, n + 1):
        if shape == "path":
            par[i] = i - 1
        elif shape == "star":
            par[i] = 1
        elif shape == "caterpillar":
            par[i] = i - 1 if i <= n // 2 or i == 2 else rng.randint(1, n // 2)
        elif shape == "broom":
            par[i] = i - 1 if i <= 2 * n // 3 else 2 * n // 3
        elif shape == "deep":
            par[i] = rng.randint(max(1, i - 3), i - 1)
        else:
            par[i] = rng.randint(1, i - 1)
    deg = [0] * (n + 1)
    for i in range(2, n + 1):
        deg[par[i]] += 1
        deg[i] += 1
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    non_leaves = [perm[u - 1] for u in range(1, n + 1) if deg[u] >= 2]
    k = rng.choice([1, 2, n, rng.randint(1, n), rng.randint(1, min(n, 20))])
    print(n, k, rng.choice(non_leaves))
    edges = [(perm[par[i] - 1], perm[i - 1]) for i in range(2, n + 1)]
    rng.shuffle(edges)
    print("\n".join(f"{a} {b}" if rng.random() < 0.5 else f"{b} {a}" for a, b in edges))
