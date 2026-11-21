#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(4, 14)
    p = list(range(1, n + 1))
    rng.shuffle(p)
    print(n)
    print(*p)
