#!/usr/bin/env python3
"""gen.py seed size | --max: CCO 2019 Card Scoring, 2 <= k <= 4, n <= 1e6 cards in 1..=n.
--max: k = 3, n = 1e6 cards over a thousand values, so each value's Li Chao tree holds about a
thousand curves."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 10 ** 6 if MAX else r.randint(1, max(1, int(sys.argv[2])))
k = 3 if MAX else r.randint(2, 4)
v = 1000 if MAX else r.choice([1, 2, n])
print(k, n)
print(*[r.randint(1, v) for _ in range(n)])
