#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    a = [-1] * (n + 1)
    for i in range(1, n):
        if rng.random() < 0.35:
            a[i] = rng.randint(0, n)
    print(n)
    print(*a)
