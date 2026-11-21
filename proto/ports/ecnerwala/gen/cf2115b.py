#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, 10)
    q = rng.randint(1, 12)
    a = [rng.randint(1, 15) for _ in range(n)]
    b = list(a)
    ops = []
    for _ in range(q):
        x = rng.randint(1, n)
        y = rng.randint(1, n)
        z = rng.randint(1, n)
        ops.append((x, y, z))
        b[z - 1] = min(b[x - 1], b[y - 1])
    if rng.random() < 0.35:
        b[rng.randrange(n)] = rng.randint(1, 15)
    print(n, q)
    print(*b)
    for x, y, z in ops:
        print(x, y, z)
