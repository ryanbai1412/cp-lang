#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    if r.random() < 0.35:
        w = r.randint(1, 124)
    else:
        w = r.randint(125, 100000)
    print(w)
