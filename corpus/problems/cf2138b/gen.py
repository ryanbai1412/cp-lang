#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 50)
    q = r.randint(1, 10 if size == 0 else 50)
    p = list(range(1, n + 1))
    r.shuffle(p)
    print(n, q)
    print(*p)
    for _ in range(q):
        l = r.randint(1, n)
        rr = r.randint(l, n)
        print(l, rr)
