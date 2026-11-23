#!/usr/bin/env python3
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); print(n)
    p = r.random()
    A = [y for y in range(n) if r.random() < p]
    a = []
    for k in range(1, n + 1):
        seen = {y // k for y in A}
        m = 0
        while m in seen: m += 1
        a.append(m)
    print(*a)
