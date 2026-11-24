#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SAC '22 CC1 P3. An N x M grid of X, W and . (N, M, Q <= 100), then
Q actions, 1 (drop and settle) or 2 (print). --max: N = M = Q = 100."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 100 if MAX else min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 100)
n = 100 if MAX else r.randint(1, size)
m = 100 if MAX else r.randint(1, size)
q = 100 if MAX else r.randint(1, size)
weights = [r.random() for _ in range(3)]
print(n, m)
for _ in range(n):
    print("".join(r.choices("XW.", weights=weights, k=m)))
print(q)
for i in range(q):
    print(2 if i == q - 1 or r.random() < 0.3 else 1)
