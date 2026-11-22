#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    k = r.randint(1, 8 if size == 0 else 40)
    x = r.randint(1, (1 << (k + 1)) - 1)
    print(k, x)
