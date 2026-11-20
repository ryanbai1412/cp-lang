#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 10)
print(T)
for _ in range(T):
    mx = 20 if size == 0 else 10**9
    print(r.randint(1, mx), r.randint(1, mx))
