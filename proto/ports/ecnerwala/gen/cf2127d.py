#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    edges = set()
    mode = rng.randint(0, 3)
    if mode == 0:
        # caterpillar
        spine = rng.randint(1, max(1, n // 2))
        for i in range(1, spine):
            edges.add((i, i + 1))
        for i in range(spine + 1, n + 1):
            p = rng.randint(1, spine)
            edges.add((min(p, i), max(p, i)))
    else:
        for i in range(2, n + 1):
            p = rng.randint(1, i - 1)
            edges.add((p, i))
        if mode == 3 and n >= 3:
            u, v = rng.sample(range(1, n + 1), 2)
            edges.add((min(u, v), max(u, v)))
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    el = [(perm[u - 1], perm[v - 1]) for u, v in edges]
    print(n, len(el))
    for u, v in el:
        print(u, v)
