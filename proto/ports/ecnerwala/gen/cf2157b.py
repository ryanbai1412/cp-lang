#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, max(2, size * 5))
    x = rng.randint(-n - 2, n + 2)
    y = rng.randint(-n - 2, n + 2)
    s = "".join(rng.choice("48") for _ in range(n))
    print(n, x, y)
    print(s)
