#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.choice([1, 3, 5, 7, 9, 11])
    print(n)
    print(*[rng.randint(1, 5) for _ in range(n)])
