#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100"]
    big = True
else:
    big = False
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if big:
    n, cap = 100, r.randint(10**8, 10**9)
    items = [(r.randint(1, 10**9), 1000 - r.randint(0, 9)) for _ in range(n)]
else:
    n = r.randint(1, min(size, 100))
    cap = r.randint(1, 30) if size < 50 else r.randint(1, 10**9)
    hi = 12 if size < 50 else 10**9
    items = [(r.randint(1, hi), r.randint(1, 9 if size < 50 else 1000)) for _ in range(n)]
print(n, cap)
for w, v in items:
    print(w, v)
