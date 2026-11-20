#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, 10 ** rng.randint(1, 6))
    rk, ck = rng.randint(0, n), rng.randint(0, n)
    rd, cd = rng.randint(0, n), rng.randint(0, n)
    while (rk, ck) == (rd, cd):
        rd, cd = rng.randint(0, n), rng.randint(0, n)
    print(n, rk, ck, rd, cd)
