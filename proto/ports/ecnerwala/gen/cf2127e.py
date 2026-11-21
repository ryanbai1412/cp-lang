#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(2, 16)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 100) for _ in range(n)])
    print(*[0 if rng.random() < 0.4 else rng.randint(1, k) for _ in range(n)])
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
