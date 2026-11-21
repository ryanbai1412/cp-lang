#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 12)
    x = rng.randint(1, 12)
    a = [rng.randint(1, 6)]
    for _ in range(n - 1):
        a.append(a[-1] * rng.randint(1, 3))
    b = [v * (x if rng.random() < 0.5 else 1) for v in a]
    print(n)
    print(*b)
