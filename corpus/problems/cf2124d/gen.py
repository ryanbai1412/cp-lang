#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # One test at the limit (the sum of n over tests is at most 2e5): n = 2e5, k = n / 2,
    # values in 1..n, so the threshold sits mid-array and every pass runs over all of a.
    r = random.Random(0)
    print(1); print(200000, 100000)
    print(*[r.randint(1, 200000) for _ in range(200000)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(1, 16)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, max(2, n // 2)) for _ in range(n)])
