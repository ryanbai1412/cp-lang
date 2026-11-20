#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    print(n)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    for i in range(1, n):
        p = rng.randint(0, i - 1)
        print(perm[p], perm[i])
