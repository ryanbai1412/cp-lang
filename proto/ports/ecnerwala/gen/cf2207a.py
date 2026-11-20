#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(3, 18)
    print(n)
    print("".join(rng.choice("01") for _ in range(n)))
