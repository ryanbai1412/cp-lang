#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one n = 3e5 test (sum n): runs climbing by 1 from random starts near 1e9, broken by
    # equal values and drops, so most removals happen and the linked list relinks deep chains
    g = random.Random(1)
    n, a = 300_000, []
    while len(a) < n:
        v = g.randint(1, 10**9 - 2000)
        for _ in range(g.randint(1, 2000)):
            a.append(v)
            v += 1 if g.random() < 0.8 else g.choice((0, -g.randint(1, 5)))
            v = max(1, v)
    print(1)
    print(n)
    print(*a[:n])
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    print(n)
    print(*[rng.randint(1, 6) for _ in range(n)])
