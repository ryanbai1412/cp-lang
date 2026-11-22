#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, max(2, min(6, size)))
    perm = list(range(1, n * n + 2))
    rng.shuffle(perm)
    print(n)
    print(*perm)
