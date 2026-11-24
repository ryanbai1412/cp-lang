#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 1e4 and sum n = 2e5: one n = 1e5 test and 9999 smaller ones, m at the 1e9 bound or
    # just around the 2l + r / l + 2r / 3l thresholds, k at the ends, the middle and random
    g = random.Random(1)
    tests = [(10**5, 10**9, 1)]
    left = 10**5
    for i in range(9999):
        n = left // (9999 - i)
        left -= n
        k = g.choice((1, n, (n + 1) // 2, g.randint(1, n)))
        l, r = sorted((k - 1, n - k))
        m = g.choice((10**9, max(1, l + 2 * r - 1 + g.randint(-1, 1)), max(1, 3 * l - 1 + g.randint(-1, 1)),
                      g.randint(1, 3 * n)))
        tests.append((n, m, k))
    print(len(tests))
    for t in tests:
        print(*t)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(1, 1000)
    m = rng.randint(0, 3000)
    k = rng.randint(1, n)
    print(n, m, k)
