#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, 16)
    print(n)
    print(*[rng.randint(0, (1 << 20) - 1) for _ in range(n)])
    print(*[rng.randint(0, (1 << 20) - 1) for _ in range(n)])
    print("".join(rng.choice("01") for _ in range(n)))
