#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # statement max: sum n^2 <= 9e6 with n <= 3000, so one case n = 3000, every label '?'
    # (both edges tried at each merge). Shape: two paths of n/2 from the root (the slowest of
    # path, star, random, broom and two-path trees), so every merge sees a deep depth table.
    n = 3000
    parent = [0, 0, 1, 1] + [v - 2 for v in range(4, n + 1)]
    kids = [[] for _ in range(n + 1)]
    for v in range(2, n + 1):
        kids[parent[v]].append(v)
    print(1); print(n); print("?" * (n - 1))
    for v in range(1, n + 1):
        print(len(kids[v]), *kids[v])
    sys.exit()
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
