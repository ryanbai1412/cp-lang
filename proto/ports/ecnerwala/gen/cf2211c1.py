#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    k = rng.randint(1, n)
    print(n, k)
    a = list(range(1, n + 1))
    rng.shuffle(a)
    b = a[:]
    if rng.random() < 0.6 and n - k < k:
        mid = b[n - k:k]
        rng.shuffle(mid)
        b[n - k:k] = mid
    if rng.random() < 0.4:
        b[rng.randint(0, n - 1)] = rng.randint(1, n)
    for i in range(n):
        if rng.random() < 0.35:
            b[i] = -1
    print(*a)
    print(*b)
