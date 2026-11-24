#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # x in the middle, hospitality rising away from x on both sides: every house is a record
    rng = random.Random(1)
    n, k = 200_000, 10**9
    x = n // 2
    h = [10**9 - 4000 * abs(i - x) - rng.randint(0, 3) for i in range(1, n + 1)]
    h = [10**9 - v + 1 for v in h]
    print(1)
    print(n, k, x)
    print(*h)
    print(*[rng.randint(1, k) for _ in range(n - 1)])
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    k = rng.randint(1, 40)
    x = rng.randint(1, n)
    print(n, k, x)
    print(*[rng.randint(1, 25) for _ in range(n)])
    print(*[rng.randint(0, 30) for _ in range(n - 1)])
