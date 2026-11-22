#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    l = rng.randint(2, 25)
    n = rng.randint(1, min(l, 10))
    m = rng.randint(1, 10)
    a = sorted(rng.sample(range(1, l + 1), n))
    print(n, m, l)
    print(*a)
