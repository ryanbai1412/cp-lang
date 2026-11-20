#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(4, 10 if size == 0 else 30)
    k = r.randint(1, 10)
    all_edges = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    r.shuffle(all_edges)
    m = r.randint(0, min(len(all_edges), 2 * n))
    print(n, m, k)
    for u, v in all_edges[:m]:
        if r.random() < 0.5:
            u, v = v, u
        print(u, v)
