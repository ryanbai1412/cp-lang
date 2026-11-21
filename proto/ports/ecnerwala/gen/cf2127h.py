#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    # Generate a simple series-parallel graph (no K4 minor) by tree + ear subdivisions / parallel-free additions
    n_target = rng.randint(3, 12)
    # Start from a single edge (1, 2), repeatedly subdivide an edge or attach a leaf or triangle on an edge
    edges = [(1, 2)]
    cur_n = 2
    while cur_n < n_target:
        cur_n += 1
        op = rng.randint(0, 2)
        if op == 0:
            # attach leaf
            u = rng.randint(1, cur_n - 1)
            edges.append((u, cur_n))
        elif op == 1:
            # subdivide existing edge
            idx = rng.randrange(len(edges))
            u, v = edges.pop(idx)
            edges.append((u, cur_n))
            edges.append((cur_n, v))
        else:
            # add parallel 2-path (triangle on existing edge)
            u, v = rng.choice(edges)
            edges.append((u, cur_n))
            edges.append((cur_n, v))
    edges = list({(min(u, v), max(u, v)) for u, v in edges})
    perm = list(range(1, cur_n + 1))
    rng.shuffle(perm)
    print(cur_n, len(edges))
    for u, v in edges:
        print(perm[u - 1], perm[v - 1])
