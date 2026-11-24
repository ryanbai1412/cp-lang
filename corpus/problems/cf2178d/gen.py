#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n with distinct a_i up to 1e9 and m = 0: the longest attack list and
    # the summing search for the start
    g = random.Random(1)
    n = 200_000
    print(1)
    print(n, 0)
    print(*g.sample(range(1, 10**9 + 1), n))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    m = rng.randint(0, n)
    print(n, m)
    print(*rng.sample(range(1, 50), n))
