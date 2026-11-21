#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 25)
    all_e = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    r.shuffle(all_e)
    m = r.randint(0, min(len(all_e), 2 * n))
    print(n, m)
    for u, v in all_e[:m]:
        print(u, v)
    q = r.randint(1, 8 if size == 0 else 30)
    print(q)
    for _ in range(q):
        l = r.randint(1, n)
        rr = r.randint(l, n)
        k = r.randint(1, rr - l + 1)
        print(l, rr, k)
