#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 25)
    v = r.randint(1, 1000)
    edges = set()
    # connected graph
    for i in range(2, n + 1):
        p = r.randint(1, i - 1)
        edges.add((p, i))
    extra = r.randint(0, n)
    for _ in range(extra):
        a = r.randint(1, n)
        b = r.randint(1, n)
        if a != b:
            edges.add((min(a, b), max(a, b)))
    print(n, len(edges), v)
    a = [-1 if r.random() < 0.7 else r.randint(0, min(v - 1, 3)) for _ in range(n)]
    print(*a)
    for x, y in sorted(edges):
        print(x, y)
