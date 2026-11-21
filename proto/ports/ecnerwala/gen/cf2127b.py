#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(2, 12 if size == 0 else 50)
    x = r.randint(1, n)
    s = [r.choice(".#") for _ in range(n)]
    s[x - 1] = "."
    print(n, x)
    print("".join(s))
