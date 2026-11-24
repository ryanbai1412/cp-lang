#!/usr/bin/env python3
import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 5000: multiples of 2310 / p for p in 2, 3, 5, 7, 11, so the gcd is 1, no
    # value is 1 and the fewest elements reaching gcd 1 is large
    rng = random.Random(1)
    pool = sorted({k * b for b in (1155, 770, 462, 330, 210) for k in range(1, 5000 // b + 1)})
    print(1)
    print(5000)
    print(*[rng.choice(pool) for _ in range(5000)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, 12)
    print(n)
    print(*[rng.randint(1, 60) for _ in range(n)])
