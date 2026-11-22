#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 10)
    print(n)
    vals = [rng.choice([1, 50, 99, 100, 101]) for _ in range(n)]
    print(*vals)
