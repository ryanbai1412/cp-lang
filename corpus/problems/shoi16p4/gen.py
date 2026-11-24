#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SHOI 2016 Darkness Before Fantasyland. 2 <= n <= 17 cities, then
n - 1 companies, each a list of distinct non-loop roads. --max: n = 17, every company can build
every road."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 17)
n = 17 if MAX else r.randint(2, max(2, size))
pairs = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
p = 1.0 if MAX else r.random()
print(n)
for _ in range(n - 1):
    mine = [e if r.random() < 0.5 else e[::-1] for e in pairs if r.random() < p]
    r.shuffle(mine)
    print(len(mine), *(x for e in mine for x in e))
