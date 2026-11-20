#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(4, size * 5))
    q = rng.randint(1, max(3, size * 4))
    print(n, q)
    print(*[rng.randint(2, 10 ** rng.randint(1, 7)) for _ in range(n)])
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        print(l, r)
