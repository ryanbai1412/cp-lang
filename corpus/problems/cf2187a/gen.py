#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one n = 2e5 test (sum n): values up to 1e9 with many duplicates, mostly in place but with
    # random swaps, so the stable sort does full work and most elements move
    g = random.Random(1)
    n = 200_000
    a = sorted(g.choice((g.randint(1, 10**9), g.randint(1, 1000))) for _ in range(n))
    for _ in range(n // 2):
        i, j = g.randrange(n), g.randrange(n)
        a[i], a[j] = a[j], a[i]
    print(1)
    print(n)
    print(*a)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    print(n)
    a = [rng.randint(1, 30) for _ in range(n)]
    if rng.random() < 0.25: a.sort()
    print(*a)
