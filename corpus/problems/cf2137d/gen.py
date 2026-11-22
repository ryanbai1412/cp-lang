#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 40)
    if r.random() < 0.5:
        b = []
        rem = n
        while rem > 0:
            k = r.randint(1, rem)
            b.extend([k] * k)
            rem -= k
        r.shuffle(b)
    else:
        b = [r.randint(1, n) for _ in range(n)]
    print(n)
    print(*b)
