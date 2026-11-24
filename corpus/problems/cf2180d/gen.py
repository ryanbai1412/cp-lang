#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n = 2e6 on random distinct centres up to 1e9
    g = random.Random(1)
    n = 2_000_000
    print(1)
    print(n)
    print(*sorted(g.sample(range(10**9 + 1), n)))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    xs = sorted(rng.sample(range(1, 100), n))
    print(n)
    print(*xs)
