#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one n = 2e5 test (sum n) of values at +-1e9, so the running sums reach 4e14
    g = random.Random(1)
    n = 200_000
    print(1)
    print(n)
    print(*[g.choice((10**9, -10**9, g.randint(-10**9, 10**9))) for _ in range(n)])
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 15)
    print(n)
    print(*[rng.randint(-50, 50) for _ in range(n)])
