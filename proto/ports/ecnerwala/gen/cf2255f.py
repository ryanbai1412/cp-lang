#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, max(2, min(40, size * 5)))
    print(n)
    print(*[rng.randint(0, 998244352) for _ in range(n)])
