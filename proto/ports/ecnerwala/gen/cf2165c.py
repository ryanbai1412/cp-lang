#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    q = rng.randint(1, 8)
    print(n, q)
    print(*[rng.randint(0, 100000) for _ in range(n)])
    for _ in range(q):
        print(rng.randint(0, 100000))
