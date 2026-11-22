#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(3, size * 5))
    l = rng.randint(1, 20)
    r = rng.randint(l, l + 20)
    print(n, l, r)
    print(*[rng.randint(1, 40) for _ in range(n)])
