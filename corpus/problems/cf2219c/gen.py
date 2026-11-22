#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 40)
    s = [r.choice("01") for _ in range(n)]
    if "1" not in s:
        s[r.randrange(n)] = "1"
    print(n)
    print("".join(s))
    for i in range(2, n + 1):
        p = r.randint(1, i - 1)
        print(p, i)
