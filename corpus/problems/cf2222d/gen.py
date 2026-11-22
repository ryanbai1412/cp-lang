#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    print(n)
    print(*[rng.randint(-20, 20) for _ in range(n)])
