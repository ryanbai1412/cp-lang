#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    print(n)
    a = [rng.randint(1, 30) for _ in range(n)]
    if rng.random() < 0.25: a.sort()
    print(*a)
