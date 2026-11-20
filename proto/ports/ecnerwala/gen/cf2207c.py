#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    h = rng.randint(1, 30)
    print(n, h)
    print(*[rng.randint(0, h) for _ in range(n)])
