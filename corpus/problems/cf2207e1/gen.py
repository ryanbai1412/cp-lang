#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n: a valid non-increasing a (n - i <= a_i <= a_{i-1}), so b is built in full
    g = random.Random(1)
    n = 200_000
    a, hi = [], n
    for i in range(1, n + 1):
        hi = g.randint(max(n - i, hi - 2), hi)
        a.append(hi)
    print(1)
    print(n)
    print(*a)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    print(n)
    if rng.random() < 0.7:
        a = [0] * n
        cur = n
        for i in range(n):
            lo = n - i - 1
            a[i] = rng.randint(lo, cur)
            cur = a[i]
        print(*a)
    else:
        print(*[rng.randint(0, n + 1) for _ in range(n)])
