#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    r = rng.randint(2, 6)
    print(n, r)
    print(*[rng.randint(0, 20) for _ in range(n)])
    print(*[rng.randint(1, 50) for _ in range(n)])
