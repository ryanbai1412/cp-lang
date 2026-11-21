#!/usr/bin/env python3
import random, sys

r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = 3 if size == 0 else 2
print(T)
for _ in range(T):
    n = r.randint(2, 12 if size == 0 else min(845, 25 * size + 10))
    p = list(range(1, n + 1))
    r.shuffle(p)
    print(n)
    print(*p)
