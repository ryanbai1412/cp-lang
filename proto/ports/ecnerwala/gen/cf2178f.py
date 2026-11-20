#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 16)
    print(n)
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
