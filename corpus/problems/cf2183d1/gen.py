#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one n = 2e5 tree (sum n): a 1e5-vertex path down from root 1 ending in a 1e5-leaf star,
    # so one layer holds half the tree under one shared parent and the rest is 1e5 deep;
    # labels and edge order shuffled
    g = random.Random(1)
    n = 200_000
    lab = list(range(2, n + 1))
    g.shuffle(lab)
    lab = [1] + lab
    edges = [(lab[i - 1], lab[i]) for i in range(1, 100_000)]
    edges += [(lab[99_999], lab[i]) for i in range(100_000, n)]
    g.shuffle(edges)
    print(1)
    print(n)
    for u, v in edges:
        print(*((u, v) if g.random() < 0.5 else (v, u)))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 15)
    print(n)
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
