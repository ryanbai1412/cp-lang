#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    q = rng.randint(1, 10)
    print(n, q)
    print(*[rng.randint(-20, 20) for _ in range(n)])
    print(*[rng.randint(-20, 20) for _ in range(q)])
