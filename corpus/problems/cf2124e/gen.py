#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    a = [rng.randint(1, 30) for _ in range(n)]
    if rng.random() < 0.7 and sum(a) % 2 == 1:
        a[0] += 1
    print(n)
    print(*a)
