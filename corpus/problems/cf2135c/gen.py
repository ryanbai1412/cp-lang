#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = 2e5, m = 4e5, V = 1e9: a Hamiltonian path cut into blocks; the
    # extra edges stay inside blocks and close only even cycles, so the path edges between blocks
    # are bridges (a deep DFS) and each block is one bipartite component. A third of the blocks
    # hold one fixed weight repeated, so the answer is a large power of V.
    r = random.Random(2135)
    n, m, V = 200_000, 400_000, 10**9
    order = list(range(1, n + 1))
    r.shuffle(order)
    edges = set()
    for i in range(n - 1):
        edges.add((min(order[i], order[i + 1]), max(order[i], order[i + 1])))
    block = 40
    a = [-1] * (n + 1)
    for s in range(0, n, block):
        if (s // block) % 3 == 0:
            val = r.randint(0, V - 1)
            for i in r.sample(range(s, min(n, s + block)), 3):
                a[order[i]] = val
    while len(edges) < m:
        s = r.randrange(0, n, block)
        e = min(n, s + block)
        i = r.randrange(s, e)
        j = i + 2 * r.randint(1, block // 2) + 1
        if j < e:
            u, v = order[i], order[j]
            edges.add((min(u, v), max(u, v)))
    el = list(edges)
    r.shuffle(el)
    out = ["1", f"{n} {m} {V}", " ".join(map(str, a[1:]))]
    out += [f"{u} {v}" if r.random() < 0.5 else f"{v} {u}" for u, v in el]
    print("\n".join(out))
    sys.exit()
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
