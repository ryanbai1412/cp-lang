#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 10)
    order = [0] + rng.sample(range(1, n), n - 1)
    nxt = {}
    for k in range(n):
        nxt[order[k]] = order[(k + 1) % n]
    grid = [["0"] * n for _ in range(n)]
    for u, v in nxt.items():
        grid[u][v] = "1"
    for u, v in nxt.items():
        lo, hi = min(u, v), max(u, v)
        for w in range(lo + 1, hi):
            if w != u and w != v and rng.random() < 0.3:
                # only add if it does not become outermost for col w
                pw = [x for x in range(n) if nxt[x] == w][0]
                if (u < w and pw < u) or (u > w and pw > u):
                    grid[u][w] = "1"
    print(n)
    for row in grid:
        print("".join(row))
