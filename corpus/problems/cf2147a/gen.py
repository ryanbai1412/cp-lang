#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 1e4 pairs up to 1e9, both orders and the extremes.
    r = random.Random(2147)
    print(10_000)
    for i in range(10_000):
        x, y = [(1, 1), (10**9, 10**9), (1, 10**9), (10**9, 1)][i] if i < 4 else (r.randint(1, 10**9), r.randint(1, 10**9))
        print(x, y)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 10)
print(T)
for _ in range(T):
    mx = 20 if size == 0 else 10**9
    print(r.randint(1, mx), r.randint(1, mx))
