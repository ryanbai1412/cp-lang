#!/usr/bin/env python3
"""gen.py seed size | --max: NOIP 2022 Barracks. A connected simple graph, n <= 5e5 cities,
n - 1 <= m <= 1e6 roads: a random tree plus extra roads, often local so that blocks and bridges
mix. --max: n = 5e5, m = 1e6."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 500_000 if MAX else r.randint(1, max(1, min(500_000, int(sys.argv[2]))))
most = min(1_000_000, n * (n - 1) // 2)
m = 1_000_000 if MAX else r.randint(n - 1, min(most, n - 1 + r.choice([0, 2, n])))
label = list(range(1, n + 1)); r.shuffle(label)
roads = set()
for i in range(1, n):
    roads.add((r.randrange(max(0, i - 3), i) if r.random() < 0.7 else r.randrange(i), i))
while len(roads) < m:
    u = r.randrange(n)
    v = min(n - 1, u + r.randint(1, 4)) if r.random() < 0.8 else r.randrange(n)
    if u != v: roads.add((min(u, v), max(u, v)))
print(n, m)
for u, v in roads: print(label[u], label[v])
