#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    print(n)
    if rng.random() < 0.7:
        a = [0] * n
        cur = n
        for i in range(n):
            lo = n - i - 1
            a[i] = rng.randint(lo, cur)
            cur = a[i]
        print(*a)
    else:
        print(*[rng.randint(0, n + 1) for _ in range(n)])
