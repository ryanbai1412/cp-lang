#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2020 Dish. T <= 10 tests, n <= 500, n - 2 <= m <= 5000,
k <= 5000, d_i >= 1 summing to m k. --max: ten tests at n = 500, k = 5000: seven at m = n - 2
(the subset-sum case; solvable ones built from two groups of (size - 1) k grams, the rest
random) and three at m = 5000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])

def parts(total, cnt):
    cuts = sorted(r.sample(range(1, total), cnt - 1)) if cnt > 1 else []
    return [b - a for a, b in zip([0] + cuts, cuts + [total])]

tests = []
for t in range(10 if MAX else r.randint(1, 4)):
    n = 500 if MAX else r.randint(1, max(1, min(500, S // 2)))
    k = 5000 if MAX else r.randint(1, max(1, min(5000, S)))
    kind = (0 if t < 4 else 1 if t < 7 else 2) if MAX else r.randrange(3)
    if kind == 0 and n >= 4:
        s = r.randint(2, n - 2)
        d = parts((s - 1) * k, s) + parts((n - s - 1) * k, n - s)
        r.shuffle(d); m = n - 2
    elif kind == 1 and n >= 3 and (n - 2) * k >= n:
        m = n - 2; d = parts(m * k, n)
    else:
        m = 5000 if MAX else r.randint(max(1, n - 1), max(n - 1, min(5000, S)))
        if m * k < n:
            k = -(-n // m)
        d = parts(m * k, n)
    tests.append(f"{n} {m} {k}\n{' '.join(map(str, d))}")
print(len(tests))
print("\n".join(tests))
