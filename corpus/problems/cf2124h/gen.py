#!/usr/bin/env python3
import sys, random

if sys.argv[1:] == ["--max"]:
    # sum n^2 <= 15000^2: one test of n = 15000, half the values from a small set (long runs of
    # equal values and deep nesting), half spread over 1..n.
    rng = random.Random(2124)
    n = 15000
    print(1)
    print(n)
    print(*[rng.randint(1, 40) if rng.random() < 0.5 else rng.randint(1, n) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, 18)
    print(n)
    print(*[rng.randint(1, n) for _ in range(n)])
