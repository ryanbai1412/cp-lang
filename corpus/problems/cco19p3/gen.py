#!/usr/bin/env python3
"""gen.py seed size | --max: CCO 2019 Winter Driving. N <= 2e5 cities, a_i <= 1e4, parents
p_2..p_N, every city with at most 36 roads. --max: N = 2e5, a hub with 36 branches of nearly
equal weight (so the subset-sum split is the full meet in the middle), each branch a long path
with side twigs, a_i near 1e4."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 200_000 if MAX else max(2, min(200_000, int(sys.argv[2])))
deg = [0] * (n + 1)
par = [0] * (n + 1)
def link(v, p):
    par[v] = p; deg[v] += 1; deg[p] += 1
if MAX:
    hub = 1
    tails = []
    for v in range(2, 38):
        link(v, hub); tails.append(v)
    for v in range(38, n + 1):
        b = (v - 38) % 36
        if r.random() < 0.1 and deg[tails[b]] < 36 and par[tails[b]] != hub:
            link(v, par[tails[b]] if deg[par[tails[b]]] < 36 else tails[b])
        else:
            link(v, tails[b]); tails[b] = v
    a = [r.randint(9_990, 10_000) for _ in range(n)]
else:
    for v in range(2, n + 1):
        p = r.randint(1, v - 1)
        while deg[p] >= 36:
            p = r.randint(1, v - 1)
        link(v, p)
    a = [r.randint(1, 10_000) for _ in range(n)]
print(n)
print(*a)
print(*par[2:])
