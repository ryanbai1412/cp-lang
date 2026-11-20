#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 8 if size == 0 else 30)
    q = r.randint(1, 8 if size == 0 else 30)
    print(n, q)
    print(*[r.randint(0, 1000 if size == 0 else 10**9) for _ in range(n)])
    for _ in range(q):
        print(r.randint(0, 1000 if size == 0 else 10**9))
