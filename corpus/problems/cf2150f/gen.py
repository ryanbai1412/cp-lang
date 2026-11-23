import random, sys

# `--max`: one test at n = 200 (sum n^2 <= 4e4), a long-diameter tree with few
# extra edges, so most pairs need an operation-2 path of ~half the diameter.
MAX = sys.argv[1] == "--max"
seed, size = (0, 0) if MAX else (int(sys.argv[1]), int(sys.argv[2]))
rng = random.Random(seed)
t = 1 if MAX else rng.randint(1, 3)
print(t)
for _ in range(t):
    n = 200 if MAX else rng.randint(3, max(3, 2 * size + 3))
    edges = set()
    # random spanning tree
    for v in range(2, n + 1):
        u = v - 1 if MAX and rng.random() < 0.8 else rng.randint(1, v - 1)
        edges.add((u, v))
    extra = n // 10 if MAX else rng.randint(0, min(n * (n - 1) // 2 - (n - 1), 2 * n))
    for _ in range(extra):
        u, v = sorted(rng.sample(range(1, n + 1), 2))
        edges.add((u, v))
    elist = list(edges)
    rng.shuffle(elist)
    print(n, len(elist))
    for u, v in elist:
        if rng.randint(0, 1):
            u, v = v, u
        print(u, v)
