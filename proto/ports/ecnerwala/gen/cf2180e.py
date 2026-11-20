#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    l = rng.randint(1, 100000)
    r = l + rng.randint(0, 100000)
    print(l, r)
