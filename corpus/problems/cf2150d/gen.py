#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at n = 2e5 (sum n <= 2e5), every a_i within 10 of 1e9: the largest products mod p
    rng = random.Random(0)
    n = 200_000
    print(1); print(n); print(*[10**9 - rng.randint(0, 10) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, max(3, size * 4))
    print(n)
    print(*[rng.randint(0, 1000000) for _ in range(n)])
