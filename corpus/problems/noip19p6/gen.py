"""noip19p6 generator: T <= 5 trees of n <= 299995 vertices, given as n - 1 edges."""
import random
import sys


def tree(rng, n, shape):
    if shape == "path":
        par = [i - 1 for i in range(2, n + 1)]
    elif shape == "binary":
        par = [i // 2 for i in range(2, n + 1)]
    else:
        par = [rng.randint(max(1, i - 5), i - 1) if shape == "deep" else rng.randint(1, i - 1) for i in range(2, n + 1)]
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    edges = [(perm[p - 1], perm[i - 1]) if rng.random() < 0.5 else (perm[i - 1], perm[p - 1])
             for i, p in zip(range(2, n + 1), par)]
    rng.shuffle(edges)
    print(n)
    for u, v in edges:
        print(u, v)


if sys.argv[1] == "--max":
    rng = random.Random(0)
    print(5)
    for shape in ("path", "binary", "deep", "random", "deep"):
        tree(rng, 299995, shape)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    tree(rng, rng.randint(2, max(2, min(299995, size))), rng.choice(["path", "binary", "deep", "random"]))
