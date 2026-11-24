#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One tree with n = 3e5: a long spine with short bushy branches, so there are many leaf
    # layers to peel and the degrees keep changing.
    rng = random.Random(2165)
    n = 300_000
    print(1)
    print(n)
    edges = []
    for i in range(2, n + 1):
        p = i - 1 if rng.random() < 0.5 else rng.randint(max(1, i - 8), i - 1)
        edges.append((p, i) if rng.random() < 0.5 else (i, p))
    rng.shuffle(edges)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    print("\n".join(f"{perm[u - 1]} {perm[v - 1]}" for u, v in edges))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 16)
    print(n)
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
