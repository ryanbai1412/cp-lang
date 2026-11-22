#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    print(rng.randint(1, 9))
