#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 1e5 tests with l, r <= 1e15: aligned blocks (both ends on 2^b), equal halves around a
    # high split bit, single points, the full [1, 1e15] range and random pairs
    g = random.Random(1)
    L = 10**15
    print(10**5)
    for i in range(10**5):
        kind = i % 5
        if kind == 0:
            b = g.randint(0, 48)
            lo = g.randint(1, L >> b >> 1) << b
            hi = lo + (g.randint(1, max(1, (L - lo) >> b >> 1)) << b) - 1
        elif kind == 1:
            b = g.randint(1, 49)
            mid = g.randint(1, L >> b) << b
            h = g.randint(1, min(mid - 1, L - mid)) if min(mid - 1, L - mid) > 0 else 0
            lo, hi = mid - h, mid + h - 1
        elif kind == 2:
            lo = hi = g.randint(1, L)
        elif kind == 3:
            lo, hi = 1, L
        else:
            lo, hi = sorted((g.randint(1, L), g.randint(1, L)))
        lo, hi = max(1, lo), min(L, hi)
        if hi < lo:
            lo, hi = hi, lo
        print(lo, hi)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    l = rng.randint(1, 100000)
    r = l + rng.randint(0, 100000)
    print(l, r)
