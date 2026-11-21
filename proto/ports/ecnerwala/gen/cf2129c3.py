#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 4)
print(T)
for _ in range(T):
    n = r.randint(2, 10 if size == 0 else 40)
    s = [r.choice("()") for _ in range(n)]
    if "(" not in s: s[0] = "("
    if ")" not in s: s[-1] = ")"
    print("".join(s))
