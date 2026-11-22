#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 40)
    print(n)
    print(*[r.randint(1, min(n, 6)) for _ in range(n)])
