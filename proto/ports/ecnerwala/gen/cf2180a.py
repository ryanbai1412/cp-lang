#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 10)
print(T)
for _ in range(T):
    l = rng.randint(1, 1000)
    a = rng.randint(0, l - 1)
    b = rng.randint(1, 1000)
    print(l, a, b)
