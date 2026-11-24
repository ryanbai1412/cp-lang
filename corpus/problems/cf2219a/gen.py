#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 100 tests at q = 1e8 with p as close to 1e8 as keeps 2(p + 2q) + 1 prime: no divisor
    # exists, so each test scans all odd candidates up to the square root and prints -1
    def prime(v):
        return v > 1 and all(v % d for d in range(3, int(v**0.5) + 1, 2)) and v % 2
    tests, p = [], 10**8
    while len(tests) < 100:
        if prime(2 * (p + 2 * 10**8) + 1):
            tests.append((p, 10**8))
        p -= 1
    print(len(tests))
    for p, q in tests:
        print(p, q)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 10)
print(T)
for _ in range(T):
    if rng.random() < 0.5:
        n = rng.randint(1, 30)
        m = rng.randint(1, 30)
        q = rng.randint(0, n * m + min(n, m))
        p = 2 * n * m + n + m - 2 * q
        if p >= 0:
            print(p, q)
            continue
    print(rng.randint(0, 200), rng.randint(0, 200))
