#!/usr/bin/env python3
import sys, random

if sys.argv[1:] == ["--max"]:
    # Only sum m is bounded: t = 1e4 tests of n near 2e5, the first with m = 1e5 and the rest
    # sharing another 1e5 of m.
    rng = random.Random(2127)
    t = 10_000
    print(t)
    print(200_000, 100_000)
    for _ in range(t - 1):
        print(rng.randint(199_000, 200_000), rng.randint(0, 20))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 12)
    m = rng.randint(0, 25)
    print(n, m)
