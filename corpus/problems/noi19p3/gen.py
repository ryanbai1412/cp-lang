#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2019 Sequence. T <= 10 tests, each n k l with 1 <= l <= k <= n,
then a and b (values 1..1e9; sum n <= 1e6). Small: T <= 3, n <= size + 1, values often in 1..5
so ties are common. --max: T = 5 tests of n = 2e5, k = 1e5, l = 1 (the most demotion rounds)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
T = 5 if MAX else r.randint(1, 3)
out = [str(T)]
for _ in range(T):
    n = 200_000 if MAX else r.randint(1, S + 1)
    k = n // 2 if MAX else r.randint(1, n)
    l = 1 if MAX else r.randint(1, k)
    hi = 10**9 if MAX or r.random() < 0.4 else 5
    out.append(f"{n} {k} {l}")
    out.append(" ".join(str(r.randint(1, hi)) for _ in range(n)))
    out.append(" ".join(str(r.randint(1, hi)) for _ in range(n)))
print("\n".join(out))
