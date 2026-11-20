#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 8)
    m = rng.randint(2, 8)
    print(n, m)
    for _ in range(n):
        print("".join(rng.choice("01") for _ in range(m)))
