#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 50 trees of n = 1000 (no sum bound): deep brooms (large beauty, long binary search),
    # wide shallow trees (many levels in the subset sum) and random trees, k across 0..n.
    r = random.Random(2138)
    print(50)
    for t in range(50):
        n = 1000
        shape = t % 4
        if shape == 0:
            depth = r.randint(300, 900)
            par = [i - 1 if i <= depth else r.randint(depth - 50, depth) for i in range(2, n + 1)]
        elif shape == 1:
            par = [max(1, (i - 1) // 2 + r.randint(-1, 0)) if i > 2 else 1 for i in range(2, n + 1)]
        elif shape == 2:
            par = [r.randint(max(1, i - 40), i - 1) for i in range(2, n + 1)]
        else:
            par = [r.randint(1, i - 1) for i in range(2, n + 1)]
        k = [0, n, n // 2, r.randint(0, n)][t % 4 if t < 8 else r.randint(2, 3)]
        print(n, k)
        print(*par)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 10 if size == 0 else 35)
    k = r.randint(0, n)
    print(n, k)
    print(*[r.randint(1, i - 1) for i in range(2, n + 1)])
