#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 6 if size == 0 else r.choice([18, 60]))
    k = r.randint(1, 3)
    c = r.randint(1, (1 << n) - 1)
    print(n, k, c)
