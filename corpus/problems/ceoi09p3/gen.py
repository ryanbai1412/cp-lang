#!/usr/bin/env python3
"""CEOI 2009 Photo (official bounds): n <= 100, A <= 2e5, 0 <= x <= 3e6, 1 <= y <= A."""
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 100, A = 2e5, points anywhere in x 0..=3e6, y 1..=A
    sys.argv[1:] = ['0', '1000']
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
C = 3000000
if size >= 1000:
    n = 100
    mode = int(sys.argv[1]) % 5
    A = [200000, 200000, 1, 123456, 200000][mode]
    pts = [[(r.randint(0, C), r.randint(1, A)) for _ in range(n)],
           [(r.randint(0, 2000), r.randint(1, A)) for _ in range(n)],
           [(r.randint(0, C), 1) for _ in range(n)],
           [(i * 30000, r.randint(1, 3000)) for i in range(n)],
           [(r.choice([0, C]), r.randint(1, A)) for _ in range(n)]][mode]
else:
    n = r.randint(1, 10)
    cx, cy = r.choice([(10, 10), (100, 30), (C, 200000)])
    A = r.choice([1, r.randint(1, 50), r.randint(1, 200000)])
    pts = [(r.randint(0, cx), r.randint(1, min(cy, A))) for _ in range(n)]
print(n, A)
for x, y in pts:
    print(x, y)
