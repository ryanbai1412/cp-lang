#!/usr/bin/env python3
import sys, random

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5, k = n, values up to 1e9.
    rng = random.Random(2127)
    n = 200_000
    print(1)
    print(n, n)
    print(*[rng.randint(1, 10**9) for _ in range(n)])
    print(*[rng.randint(1, 10**9) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 50) for _ in range(n)])
    print(*[rng.randint(1, 50) for _ in range(n)])
