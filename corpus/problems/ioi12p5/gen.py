#!/usr/bin/env python3
"""gen.py seed size | --max: IOI 2012 Last Supper. N <= 1e5 colours, 1 <= K < N on the
scaffold, N requests. --max: N = 1e5, K = 5e4, requests cycling over K + 1 colours (a miss
almost every time) mixed with random spells over all N colours."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 100_000 if MAX else max(2, min(100_000, int(sys.argv[2])))
k = 50_000 if MAX else r.randint(1, n - 1)
req = []
while len(req) < n:
    base, span = r.randrange(n), k + 1
    if r.random() < 0.5:
        req += [(base + i % span) % n for i in range(r.randint(1, 3 * span))]
    else:
        req += [r.randrange(n) for _ in range(r.randint(1, n))]
req = req[:n]
print(n, k)
print(*req)
