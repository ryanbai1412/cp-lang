#!/usr/bin/env python3
import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5, values over 0..=n (sums near n^2)
    rng = random.Random(1)
    n = 200_000
    print(1)
    print(n)
    print(*[rng.randint(n // 2, n) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    print(n)
    print(*[rng.randint(0, 2 * n) for _ in range(n)])
