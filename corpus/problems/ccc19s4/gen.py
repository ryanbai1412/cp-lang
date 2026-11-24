#!/usr/bin/env python3
"""gen.py seed size | --max: CCC 2019 S4 Tourism. 1 <= K <= N <= 1e6, scores a_i <= 1e9, ceil(N/K)
days. --max: N = 1e6, K = 999 (N mod K != 0, about a thousand days, so an O(N K) DP is too
slow), scores mixing random values with long rising and falling runs."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 1_000_000 if MAX else max(1, min(1_000_000, int(sys.argv[2])))
k = 999 if MAX else r.randint(1, n)
a = []
while len(a) < n:
    run = r.randint(1, 3000)
    kind = r.randrange(3)
    lo = r.randint(1, 10 ** 9 - run)
    a += ([r.randint(1, 10 ** 9) for _ in range(run)] if kind == 0 else
          list(range(lo, lo + run)) if kind == 1 else list(range(lo + run, lo, -1)))
print(n, k)
print(*a[:n])
