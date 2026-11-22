#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    k = rng.randint(1, n)
    print(n, k)
    a = [rng.randint(1, min(n, 4)) for _ in range(n)]
    b = a[:]
    if rng.random() < 0.5:
        for i in range(n):
            if rng.random() < 0.3:
                b[i] = rng.randint(1, min(n, 4))
    for i in range(n):
        if rng.random() < 0.35:
            b[i] = -1
    print(*a)
    print(*b)
