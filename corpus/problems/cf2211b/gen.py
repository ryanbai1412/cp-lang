#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    x = rng.randint(0, 20)
    y = rng.randint(0, 20)
    if x == 0 and y == 0: x = 1
    print(x, y)
