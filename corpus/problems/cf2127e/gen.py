#!/usr/bin/env python3
import sys, random

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5: a deep, bushy tree, k = n / 2 colors with half the vertices lost,
    # weights near 1e9.
    rng = random.Random(2127)
    n, k = 200_000, 100_000
    print(1)
    print(n, k)
    print(*[rng.randint(10**9 - 1000, 10**9) for _ in range(n)])
    print(*[0 if rng.random() < 0.5 else rng.randint(1, k) for _ in range(n)])
    perm = [1] + rng.sample(range(2, n + 1), n - 1)
    out = []
    for i in range(2, n + 1):
        p = i - 1 if rng.random() < 0.6 else rng.randint(max(1, i - 20), i - 1)
        u, v = perm[p - 1], perm[i - 1]
        out.append(f"{u} {v}" if rng.random() < 0.5 else f"{v} {u}")
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(2, 16)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 100) for _ in range(n)])
    print(*[0 if rng.random() < 0.4 else rng.randint(1, k) for _ in range(n)])
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
