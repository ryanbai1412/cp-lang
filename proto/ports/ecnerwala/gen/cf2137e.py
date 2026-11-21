#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(2, 10 if size == 0 else 40)
    k = r.randint(1, 5 if size == 0 else 10**9)
    print(n, k)
    print(*[r.randint(0, n) for _ in range(n)])
