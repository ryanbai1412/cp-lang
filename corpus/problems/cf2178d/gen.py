#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    m = rng.randint(0, n)
    print(n, m)
    print(*rng.sample(range(1, 50), n))
