#!/usr/bin/env python3
import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 5e4, values up to 1e12 (sums near 5e16), even total and no entry above
    # half, so the two-operation construction runs
    rng = random.Random(1)
    n = 50_000
    a = [rng.randint(1, 10**12) for _ in range(n)]
    if sum(a) % 2:
        a[0] += 1 if a[0] < 10**12 else -1
    print(1)
    print(n)
    print(*a)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    a = [rng.randint(1, 30) for _ in range(n)]
    if rng.random() < 0.7 and sum(a) % 2 == 1:
        a[0] += 1
    print(n)
    print(*a)
