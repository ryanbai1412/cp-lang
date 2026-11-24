#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 500 tests with l, b at 5000 and around it (gcd 1, gcd l, and in between)
    g = random.Random(1)
    print(500)
    for i in range(500):
        l = 5000 if i % 2 else g.randint(4000, 5000)
        b = (5000, 4999, 2500, g.randint(1, 5000))[i % 4]
        print(l, g.randint(0, l - 1), b)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 10)
print(T)
for _ in range(T):
    l = rng.randint(1, 1000)
    a = rng.randint(0, l - 1)
    b = rng.randint(1, 1000)
    print(l, a, b)
