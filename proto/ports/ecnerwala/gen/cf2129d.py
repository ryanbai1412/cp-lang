#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 6 if size == 0 else 14)
    print(n)
    print(*[-1 if r.random() < 0.6 else r.randint(0, 3) for _ in range(n)])
