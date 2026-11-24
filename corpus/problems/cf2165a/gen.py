#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5, values up to 1e9.
    rng = random.Random(2165)
    n = 200_000
    print(1)
    print(n)
    print(*[rng.randint(0, 10**9) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 15)
    print(n)
    print(*[rng.randint(0, 100) for _ in range(n)])
