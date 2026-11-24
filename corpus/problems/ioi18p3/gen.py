#!/usr/bin/env python3
"""gen.py seed size | --max: IOI 2018 Werewolf. A connected graph on n <= 2e5 cities with
m <= 4e5 roads, q <= 2e5 trips "s e l r" with l <= s, e <= r, s != e. --max: n = 2e5,
m = 4e5, q = 2e5: a long random path plus random chords (deep Kruskal trees) and trips with
middling l and r so neither side's reach is trivial."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 200_000 if MAX else max(2, min(200_000, int(sys.argv[2])))
m = 400_000 if MAX else r.randint(n - 1, min(400_000, n * (n - 1) // 2, 2 * n))
q = 200_000 if MAX else r.randint(1, max(1, n))
order = list(range(n)); r.shuffle(order)
edges = set()
for i in range(1, n):
    u, v = order[i - 1], order[i]
    edges.add((min(u, v), max(u, v)))
while len(edges) < m:
    u, v = r.randrange(n), r.randrange(n)
    if u != v: edges.add((min(u, v), max(u, v)))
edges = list(edges); r.shuffle(edges)
out = [f"{n} {m} {q}"] + [f"{u} {v}" for u, v in edges]
for _ in range(q):
    while True:
        s, e = r.randrange(n), r.randrange(n)
        if s == e: continue
        l, rr = r.randint(0, s), r.randint(e, n - 1)
        if MAX:
            l, rr = max(l, s - n // 3), min(rr, e + n // 3)
        if l <= rr: break
    out.append(f"{s} {e} {l} {rr}")
sys.stdout.write("\n".join(out) + "\n")
