#!/usr/bin/env python3
import sys, random

if sys.argv[1:] == ["--max"]:
    # n = 2e5, m = n - 1: a caterpillar with a 1e5-long spine (deep for a recursive walk) and the
    # other 1e5 vertices hung as leaves, unevenly, so the factorials are large.
    rng = random.Random(2127)
    n, spine = 200_000, 100_000
    edges = [(i, i + 1) for i in range(1, spine)]
    for i in range(spine + 1, n + 1):
        p = rng.choice((1, spine, spine // 2)) if rng.random() < 0.3 else rng.randint(1, spine)
        edges.append((p, i))
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    rng.shuffle(edges)
    print(1)
    print(n, len(edges))
    print("\n".join(f"{perm[u - 1]} {perm[v - 1]}" if rng.random() < 0.5 else f"{perm[v - 1]} {perm[u - 1]}" for u, v in edges))
    sys.exit()
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
