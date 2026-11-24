#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n = sum m = 1e5, |a_i| up to 1e9 of both signs, every index marked once
    g = random.Random(1)
    n = m = 100_000
    xs = list(range(1, n + 1))
    g.shuffle(xs)
    print(1)
    print(n, m)
    print(*[g.randint(-10**9, 10**9) for _ in range(n)])
    print(*xs)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 14)
    m = rng.randint(1, n)
    print(n, m)
    print(*[rng.randint(-20, 20) for _ in range(n)])
    xs = rng.sample(range(1, n + 1), m)
    print(*xs)
