#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 4e5, values up to 1e18: a random walk in log scale with ratios around
    # 1..3 (where segment costs 2 and 3 compete) and runs of equal values.
    rng = random.Random(2159)
    n = 400_000
    a, x = [], 10**17
    for _ in range(n):
        if rng.random() < 0.1:
            a.append(a[-1] if a else x)
            continue
        x = int(x * rng.choice([1.0, 1.3, 1.6, 2.1, 2.9, 0.33, 0.5, 0.7]))
        x = min(10**18, max(1, x)) if rng.random() < 0.999 else rng.choice([1, 10**18])
        a.append(x)
    print(1)
    print(n)
    print(*a)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    print(n)
    print(*[rng.randint(1, 1000) for _ in range(n)])
