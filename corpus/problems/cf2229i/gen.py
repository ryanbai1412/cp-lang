#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, 10)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 10) for _ in range(n)])
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
