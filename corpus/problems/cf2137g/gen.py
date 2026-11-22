#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 30)
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    edges = []
    for i in range(n):
        for j in range(i + 1, n):
            if r.random() < 0.35:
                edges.append((perm[i], perm[j]))
    uncolored = list(range(1, n + 1))
    r.shuffle(uncolored)
    q = r.randint(2, 10 if size == 0 else 40)
    qs = []
    for _ in range(q):
        if uncolored and r.random() < 0.4:
            qs.append((1, uncolored.pop()))
        else:
            qs.append((2, r.randint(1, n)))
    print(n, len(edges), len(qs))
    for u, v in edges:
        print(u, v)
    for op, v in qs:
        print(op, v)
