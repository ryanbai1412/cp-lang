#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 10)
    print(n)
    words = ["".join(rng.choice("abc") for _ in range(rng.randint(1, 4))) for _ in range(n)]
    print(*words)
