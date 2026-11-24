#!/usr/bin/env python3
"""gen.py seed size | --max: CSP-S 2019 Centroid of Tree. T <= 5 trees of n <= 299995 vertices,
shuffled labels. --max: T = 5 at n = 299995: a path, a star, a caterpillar, a random tree and a
binary-ish tree (deep and shallow centroids, two-centroid splits)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
N = 299_995
shapes = ["path", "star", "caterpillar", "random", "binary"]
t = 5 if MAX else r.randint(1, 5)
print(t)
for k in range(t):
    n = N if MAX else r.randint(1, max(1, min(N, int(sys.argv[2]))))
    shape = shapes[k] if MAX else r.choice(shapes)
    def parent(i):
        if shape == "path": return i - 1
        if shape == "star": return 0
        if shape == "caterpillar": return i - 1 if i % 2 else max(0, i - 2)
        if shape == "binary": return (i - 1) // 2
        return r.randrange(i)
    label = list(range(1, n + 1)); r.shuffle(label)
    print(n)
    for i in range(1, n):
        u, v = label[parent(i)], label[i]
        print(*((u, v) if r.random() < 0.5 else (v, u)))
