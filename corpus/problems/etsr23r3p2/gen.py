#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n folds `ax ay bx by` (a != b), then q points `x y`.
import random, sys
MAX = sys.argv[1] == "--max"  # 10 folds, 1e5 points
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 10 if MAX else r.randint(1, 4)
q = 100000 if MAX else r.randint(1, 10)
def pt(): return r.randint(0, 100), r.randint(0, 100)
print(n)
for _ in range(n):
    a = pt()
    b = pt()
    while b == a: b = pt()
    print(*a, *b)
print(q)
for _ in range(q): print(*pt())
