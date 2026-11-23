#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # n = 2500 distinct random points in [-1e9, 1e9]^2: in general position (no three
    # collinear, as the statement requires) with overwhelming probability.
    sys.argv[1:] = ["0", "2500"]
seed = int(sys.argv[1])
size = int(sys.argv[2])
rng = random.Random(seed)

n = max(5, size) if size >= 100 else rng.randint(5, max(5, size))
lim = 10**9 if size >= 100 else 10**6

pts = []
# For small n, reject collinear triples explicitly; for large n in [-1e9, 1e9]^2, random points are almost surely in general position
if n <= 30:
    while len(pts) < n:
        x = rng.randint(-lim, lim)
        y = rng.randint(-lim, lim)
        ok = True
        for i in range(len(pts)):
            if pts[i] == (x, y):
                ok = False
                break
            x1, y1 = pts[i]
            for j in range(i + 1, len(pts)):
                x2, y2 = pts[j]
                if (x1 - x) * (y2 - y) == (y1 - y) * (x2 - x):
                    ok = False
                    break
            if not ok:
                break
        if ok:
            pts.append((x, y))
else:
    seen = set()
    while len(pts) < n:
        x = rng.randint(-lim, lim)
        y = rng.randint(-lim, lim)
        if (x, y) not in seen:
            seen.add((x, y))
            pts.append((x, y))

print(n)
for x, y in pts:
    print(x, y)
