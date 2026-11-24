"""yac7p4 generator: a tree of N <= 2e5 towns (N - 1 roads u v), then M <= 2e5 appearances."""
import random
import sys


def emit(rng, n, m, shape, far):
    order = list(range(1, n + 1))
    rng.shuffle(order)
    edges = []
    for i in range(1, n):
        p = i - 1 if shape == "path" else rng.randrange(max(0, i - 2), i) if shape == "long" else rng.randrange(i)
        edges.append((order[i], order[p]) if rng.random() < 0.5 else (order[p], order[i]))
    rng.shuffle(edges)
    print(n, m)
    print("\n".join(f"{u} {v}" for u, v in edges))
    ends = [order[0], order[-1]]
    print("\n".join(str(ends[i % 2] if far and rng.random() < 0.9 else rng.randint(1, n)) for i in range(m)))


if sys.argv[1] == "--max":
    emit(random.Random(0), 200000, 200000, "long", True)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
emit(rng, rng.randint(1, max(1, min(200000, size))), rng.randint(1, max(1, min(200000, size))),
     rng.choice(["path", "long", "random"]), rng.random() < 0.5)
