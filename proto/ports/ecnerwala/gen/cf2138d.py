#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 8 if size == 0 else 30)
    l = r.randint(n, n + (10 if size == 0 else 100))
    q = r.randint(1, 8 if size == 0 else 30)
    a = sorted(r.sample(range(1, l + 1), n))
    print(n, l, q)
    print(*a)
    for _ in range(q):
        i = r.randint(1, n)
        x = r.randint(i, l - (n - i))
        print(i, x)
