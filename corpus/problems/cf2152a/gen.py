#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 1000 tests of n = 100 values across 1..100.
    rng = random.Random(2152)
    print(1000)
    for _ in range(1000):
        print(100)
        print(*[rng.randint(1, 100) for _ in range(100)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, min(100, max(3, size * 5)))
    print(n)
    print(*[rng.randint(1, max(2, n)) for _ in range(n)])
