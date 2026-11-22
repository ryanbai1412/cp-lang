#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 4)
print(T)
for _ in range(T):
    n = r.randint(1, 5 if size == 0 else 20)
    lim = 1000 if size == 0 else 10**9
    rx = r.randint(-lim, lim)
    ry = r.randint(-lim, lim)
    print(n, rx, ry)
    for _ in range(n):
        print(r.randint(-lim, lim), r.randint(-lim, lim))
