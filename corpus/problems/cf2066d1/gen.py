#!/usr/bin/env python3
import random, sys

if sys.argv[1] == "--max":
    # one test at sum m = 1e4: n = c = 100, m = n c
    print(1)
    print(100, 100, 10_000)
    print(*([0] * 10_000))
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
T = 6 if S == 0 else 5
print(T)
for _ in range(T):
    hi = 6 if S == 0 else min(100, 3 * S + 5)
    n = r.randint(1, hi)
    c = r.randint(1, hi)
    m = r.randint(c, min(n * c, 2000))
    print(n, c, m)
    print(*([0] * m))
