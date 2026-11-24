#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 1e4 and sum k = 1e5, with one even k = 90000 (the bit-by-bit O(31 k) path) and
    # n of the most set bits under 1e9; the rest mix odd and even k on high-popcount n
    g = random.Random(1)
    dense = [n for n in (2**29 - 1, 2**30 - 2**27 - 1, 999_999_999, 2**29 + 2**28 - 1)
             if n <= 10**9]
    tests = [(2**30 - 2**27 - 1, 90_000)] + [(g.choice(dense), 1) for _ in range(9_998)]
    tests.append((g.choice(dense), 2))
    print(len(tests))
    for n, k in tests:
        print(n, k)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 1000000)
    k = rng.randint(1, 12)
    print(n, k)
