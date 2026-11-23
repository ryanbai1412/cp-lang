#!/usr/bin/env python3
# gen.py SEED SIZE: SIZE 0/1 small multitests; SIZE >= 2 sum of n = 1e6 (one test of 1e6, or many).
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size <= 1:
    ns = [r.randint(1, 12 if size == 0 else 80) for _ in range(r.randint(1, 5))]
else:
    total = 10 ** 6
    shape = r.randint(0, 2)
    if shape == 0:
        ns = [total]
    elif shape == 1:
        ns = [total - 1, 1]
    else:
        ns = []
        while total > 0 and len(ns) < 10 ** 4:
            k = min(total, r.randint(1, 3000))
            ns.append(k)
            total -= k
print(len(ns))
print("\n".join(map(str, ns)))
