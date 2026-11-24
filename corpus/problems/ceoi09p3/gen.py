#!/usr/bin/env python3
"""CEOI 2009 Photo (Luogu P5975 bounds): n <= 100, A <= 2e6, coordinates in 0..=1e5."""
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 100, A = 2e6, points anywhere in 0..=1e5
    sys.argv[1:] = ['0', '1000']
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
C = 100000
if size >= 1000:
    n = 100
    mode = int(sys.argv[1]) % 5
    A = [2000000, 2000000, 1, 123456, 2000000][mode]
    pts = [[(r.randint(0, C), r.randint(0, C)) for _ in range(n)],
           [(r.randint(0, 2000), r.randint(1, C)) for _ in range(n)],
           [(r.randint(0, C), r.randint(0, C)) for _ in range(n)],
           [(i * 1000, r.randint(1, 3000)) for i in range(n)],
           [(r.choice([0, C]), r.randint(0, C)) for _ in range(n)]][mode]
else:
    n = r.randint(1, 10)
    cx, cy = r.choice([(10, 10), (100, 30), (C, C)])
    A = r.choice([1, r.randint(1, 50), r.randint(1, 2000000)])
    pts = [(r.randint(0, cx), r.randint(0, cy)) for _ in range(n)]
print(n, A)
for x, y in pts:
    print(x, y)
