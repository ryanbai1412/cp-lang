import random, sys

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(3, max(3, 2 * size + 3))
    edges = set()
    # random spanning tree
    for v in range(2, n + 1):
        u = rng.randint(1, v - 1)
        edges.add((u, v))
    extra = rng.randint(0, min(n * (n - 1) // 2 - (n - 1), 2 * n))
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
