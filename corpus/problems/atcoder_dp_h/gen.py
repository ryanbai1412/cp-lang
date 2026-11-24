#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 1000:
    h = w = 1000
    wall = 50
else:
    h, w = r.randint(2, min(size + 1, 1000)), r.randint(2, min(size + 1, 1000))
    wall = 5
g = [['#' if r.randint(0, wall - 1) == 0 else '.' for _ in range(w)] for _ in range(h)]
g[0][0] = g[-1][-1] = '.'
print(h, w)
for row in g:
    print(''.join(row))
