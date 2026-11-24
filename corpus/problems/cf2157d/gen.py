#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5, the widest range 1..1e9 and estimates across it.
    rng = random.Random(2157)
    n = 200_000
    print(1)
    print(n, 1, 10**9)
    print(*[rng.randint(1, 10**9) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(3, size * 5))
    l = rng.randint(1, 20)
    r = rng.randint(l, l + 20)
    print(n, l, r)
    print(*[rng.randint(1, 40) for _ in range(n)])
