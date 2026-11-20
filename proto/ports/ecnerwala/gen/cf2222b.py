#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 14)
    m = rng.randint(1, n)
    print(n, m)
    print(*[rng.randint(-20, 20) for _ in range(n)])
    xs = rng.sample(range(1, n + 1), m)
    print(*xs)
