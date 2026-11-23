#!/usr/bin/env python3
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(2, max(2, S))
    parent = [0, 0] + [r.randint(1, v - 1) for v in range(2, n + 1)]
    kids = [[] for _ in range(n + 1)]
    for v in range(2, n + 1):
        kids[parent[v]].append(v)
    print(n)
    print("".join(r.choice("01??") for _ in range(n - 1)))
    for v in range(1, n + 1):
        r.shuffle(kids[v])
        print(len(kids[v]), *kids[v])
