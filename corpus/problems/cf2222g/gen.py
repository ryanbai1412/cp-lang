#!/usr/bin/env python3
import random, sys


def distinct_chains(n, rng):
    # centroid 1 with hanging paths of sizes 1, 2, 3, ...: the most distinct child sizes
    edges, nxt, k = [], 2, 1
    while nxt + k - 1 <= n:
        prev = 1
        for _ in range(k):
            edges.append((prev, nxt))
            prev, nxt = nxt, nxt + 1
        k += 1
    while nxt <= n:
        edges.append((1, nxt))
        nxt += 1
    return edges


def relabel(n, edges, rng):
    p = list(range(1, n + 1))
    rng.shuffle(p)
    p = [0] + p
    return [(p[u], p[v]) for u, v in edges]


if sys.argv[1] == "--max":
    rng = random.Random(1)
    n = 10**5
    tests = [
        [(i, i + 1) for i in range(1, n)],
        [(1, i) for i in range(2, n + 1)],
        distinct_chains(n, rng),
        relabel(n, distinct_chains(n, rng), rng),
        [(rng.randint(max(1, i - 3), i - 1), i) for i in range(2, n + 1)],
    ]
    out = [str(len(tests))]
    for e in tests:
        out.append(str(n))
        out.extend(f"{u} {v}" for u, v in e)
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(1, 14)
    print(n)
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
