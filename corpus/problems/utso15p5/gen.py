"""utso15p5 generator: N <= 5e4 stores, M <= 1e5 connections a b c (a != b, 1 <= c <= 1e4)."""
import random
import sys


def emit(rng, n, m, costs, connected):
    edges = []
    if connected and n > 1:
        order = list(range(1, n + 1))
        rng.shuffle(order)
        for i in range(1, n):
            edges.append((order[i], order[rng.randrange(max(0, i - 3), i)] if rng.random() < 0.5 else order[rng.randrange(i)]))
    while len(edges) < m:
        a, b = rng.randint(1, n), rng.randint(1, n)
        if a != b:
            edges.append((a, b))
    edges = edges[:m]
    rng.shuffle(edges)
    print(n, m)
    print("\n".join(f"{a} {b} {rng.randint(1, costs)}" for a, b in edges))


if sys.argv[1] == "--max":
    emit(random.Random(0), 50000, 100000, 10000, True)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(2, max(2, min(50000, size)))
m = rng.randint(1, max(1, min(100000, 2 * size)))
emit(rng, n, max(m, n - 1) if rng.random() < 0.8 else m, rng.choice([1, 3, 10000]), rng.random() < 0.8)
