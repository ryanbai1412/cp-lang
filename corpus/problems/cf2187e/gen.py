#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 14)
    print(n)
    print(*[rng.randint(0, 30) for _ in range(n)])
    print("".join(rng.choice("01") for _ in range(n)))
