#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 10 if size == 0 else 35)
    k = r.randint(0, n)
    print(n, k)
    print(*[r.randint(1, i - 1) for i in range(2, n + 1)])
