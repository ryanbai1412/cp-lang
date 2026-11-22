#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, max(4, size * 5))
    z = rng.randint(1, 20)
    xs = []
    cur = rng.randint(0, 5)
    for _ in range(n):
        cur += rng.randint(1, 10)
        xs.append(cur)
    q = rng.randint(1, max(3, size * 4))
    print(n, z)
    print(*xs)
    print(q)
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        print(l, r)
