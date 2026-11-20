#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    half = rng.randint(1, 10)
    s = []
    op = cl = 0
    for _ in range(2 * half):
        if op == half:
            s.append(")"); cl += 1
        elif op == cl:
            s.append("("); op += 1
        elif rng.random() < 0.5:
            s.append("("); op += 1
        else:
            s.append(")"); cl += 1
    print(2 * half)
    print("".join(s))
