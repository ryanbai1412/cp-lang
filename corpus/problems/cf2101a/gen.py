#!/usr/bin/env python3
import sys, random

if sys.argv[1] == "--max":
    # sum n = 1000: two tests at n = 500 (2.5e5 cells each)
    print(2)
    print(500)
    print(500)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    print(rng.randint(1, 9))
