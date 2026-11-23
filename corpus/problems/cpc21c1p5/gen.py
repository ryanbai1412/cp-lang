#!/usr/bin/env python3
# usage: gen.py seed size; prints a hidden case for tests/interactor.py (n <= 500, coordinates 1..=1e9)
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = 500 if size >= 500 else r.randint(1, max(1, min(size, 500)))
span = r.choice([10, 1000, 10**9]) if size < 500 else r.choice([1000, 10**9])
distinct = r.random() < 0.6 or size >= 500

def side():
    a, b = r.randint(1, span), r.randint(1, span)
    if r.random() < 0.1:
        a = b = r.choice([1, span])
    return min(a, b), max(a, b)

rects = []
while len(rects) < n:
    x, y = side(), side()
    k = 1 if distinct else r.randint(1, 3)
    rects.extend([(*x, *y)] * min(k, n - len(rects)))
    if distinct:
        rects = list(dict.fromkeys(rects))
r.shuffle(rects)
print(n)
for t in rects:
    print(*t)
