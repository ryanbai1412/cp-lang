#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 8 if size == 0 else 40)
    pts = list(range(1, 2 * n + 1))
    r.shuffle(pts)
    print(n)
    for i in range(n):
        a, b = pts[2 * i], pts[2 * i + 1]
        if a > b: a, b = b, a
        print(a, b)
