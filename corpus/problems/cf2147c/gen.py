#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 12 if size == 0 else 50)
    print(n)
    print("".join(r.choice("01") for _ in range(n)))
