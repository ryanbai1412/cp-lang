#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 6)
    print(n)
    for _ in range(n):
        a = rng.randint(1, 100)
        d = rng.randint(1, 50)
        b = a + d
        c = b + d
        l = c + rng.randint(0, 200)
        r = l + rng.randint(0, 200)
        print(a, b, c, l, r)
