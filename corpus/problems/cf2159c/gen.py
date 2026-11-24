#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 4e5: given pairs (both ends, or one end only), fixed points and zeros,
    # all consistent, plus 40% open indices, so the answer sums over a long range.
    rng = random.Random(2159)
    n = 400_000
    a = [-1] * (n + 1)
    idx = list(range(1, n))
    rng.shuffle(idx)
    i = 0
    while i + 1 < len(idx):
        u, v = idx[i], idx[i + 1]
        x = rng.random()
        if x < 0.4:
            pass
        elif x < 0.6:
            a[u], a[v] = v, u
        elif x < 0.75:
            a[u] = v
        elif x < 0.85:
            a[u], a[v] = u, 0
        else:
            a[u], a[v] = 0, 0
        i += 2
    print(1)
    print(n)
    print(*a)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    a = [-1] * (n + 1)
    for i in range(1, n):
        if rng.random() < 0.35:
            a[i] = rng.randint(0, n)
    print(n)
    print(*a)
