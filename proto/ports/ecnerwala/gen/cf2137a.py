#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 10)
print(T)
for _ in range(T):
    print(r.randint(1, 10 if size == 0 else 20), r.randint(1, 20))
