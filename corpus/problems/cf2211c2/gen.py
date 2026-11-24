#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n, k = 1000: even residue classes constant in a, odd ones varied, b a copy of
    # a with a third of the cells -1, so the answer is YES and every class is checked in full
    g = random.Random(1)
    n, k = 200_000, 1000
    const = [g.randint(1, n) for _ in range(k)]
    a = [const[i % k] if i % k % 2 == 0 else g.randint(1, n) for i in range(n)]
    b = [-1 if g.random() < 0.35 else x for x in a]
    print(1)
    print(n, k)
    print(*a)
    print(*b)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    k = rng.randint(1, n)
    print(n, k)
    a = [rng.randint(1, min(n, 4)) for _ in range(n)]
    b = a[:]
    if rng.random() < 0.5:
        for i in range(n):
            if rng.random() < 0.3:
                b[i] = rng.randint(1, min(n, 4))
    for i in range(n):
        if rng.random() < 0.35:
            b[i] = -1
    print(*a)
    print(*b)
