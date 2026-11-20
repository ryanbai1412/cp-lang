#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    xs = sorted(rng.sample(range(1, 100), n))
    print(n)
    print(*xs)
