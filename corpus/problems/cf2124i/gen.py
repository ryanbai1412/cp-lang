#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    x = [1]
    for _ in range(n - 1):
        if rng.random() < 0.85:
            x.append(rng.randint(2, min(n, x[-1] + 1)))
        else:
            x.append(rng.randint(1, n))
    print(n)
    print(*x)
