#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # One test at the limit (the sum of n over tests is at most 1e6): n = 1e6 values in
    # 1..2n, so every value's list and both sweeps run at full size.
    rng = random.Random(0); n = 10**6
    print(1); print(n); print(*[rng.randint(1, 2 * n) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    print(n)
    print(*[rng.randint(1, 2 * n) for _ in range(n)])
