#!/usr/bin/env python3
"""gen.py seed size | --max: NOIP 2022 Meow Meow. T tests, sum m <= 2e6, n <= 300, k <= 2n - 1,
every pattern dealt an even number of times. --max: two tests with n = 300, k = 599 and
m = 1e6, the twins of a pair dealt close together or far apart."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
t = 2 if MAX else r.randint(1, 3)
print(t)
for i in range(t):
    n = 300 if MAX else r.randint(1, max(1, min(300, int(sys.argv[2]) // 4 + 1)))
    k = 2 * n - 1 if MAX or r.random() < 0.7 else r.randint(1, 2 * n - 1)
    m = 1_000_000 if MAX else 2 * r.randint(1, max(1, min(50_000, int(sys.argv[2]))))
    deck = [c for _ in range(m // 2) for c in [r.randint(1, k)] * 2]
    reach = [8, m][i % 2] if MAX else r.choice([2, 8, m])
    for a in range(m):
        b = min(m - 1, a + r.randrange(reach))
        deck[a], deck[b] = deck[b], deck[a]
    print(n, m, k)
    print(*deck)
