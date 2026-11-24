#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 500 tests of n = 100 (no sum bound), values across 0..n.
    rng = random.Random(2157)
    print(500)
    for _ in range(500):
        print(100)
        print(*[rng.randint(0, 100) for _ in range(100)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    print(n)
    print(*[rng.randint(0, n) for _ in range(n)])
