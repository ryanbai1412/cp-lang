#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # One test at the limit (the sums of n and of q over tests are at most 5000):
    # n = q = 5000 sliders and moves over positions 1..1e9.
    r = random.Random(0); n = q = 5000; l = 10**9
    print(1); print(n, l, q); print(*sorted(r.sample(range(1, l + 1), n)))
    for _ in range(q):
        i = r.randint(1, n); print(i, r.randint(i, l - (n - i)))
    sys.exit()
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
