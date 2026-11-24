#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n, k = 3n / 4: the middle n / 2 positions lie in every window; b keeps a
    # outside, shuffles the middle and hides a third as -1, so the answer is YES
    g = random.Random(1)
    n, k = 200_000, 150_000
    a = list(range(1, n + 1))
    g.shuffle(a)
    b = a[:]
    mid = b[n - k:k]
    g.shuffle(mid)
    b[n - k:k] = mid
    b = [-1 if g.random() < 0.35 else x for x in b]
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
    a = list(range(1, n + 1))
    rng.shuffle(a)
    b = a[:]
    if rng.random() < 0.6 and n - k < k:
        mid = b[n - k:k]
        rng.shuffle(mid)
        b[n - k:k] = mid
    if rng.random() < 0.4:
        b[rng.randint(0, n - 1)] = rng.randint(1, n)
    for i in range(n):
        if rng.random() < 0.35:
            b[i] = -1
    print(*a)
    print(*b)
