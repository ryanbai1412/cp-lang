#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # One test at the limit (the sum of n over tests is at most 2000): n = 2000, h = 1e9.
    r = random.Random(0)
    print(1); print(2000, 10**9); print(*[r.randint(1, 10**9) for _ in range(2000)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    h = rng.randint(1, 30)
    print(n, h)
    print(*[rng.randint(1, h) for _ in range(n)])
