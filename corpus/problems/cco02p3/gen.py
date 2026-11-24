#!/usr/bin/env python3
"""Hidden test for CCO 2002 P3: `gen.py SEED SIZE | --max` prints `n m`, the start `r c f` (f in
NESW) and n rows of '.' (free) and 'X' (obstacle), every free cell reachable from the start.
--max is an 80 x 80 board."""
import random, sys
MAX = sys.argv[1] == "--max"
g = random.Random(0 if MAX else int(sys.argv[1]))
S = 80 if MAX else max(1, min(80, int(sys.argv[2])))
n, m = (80, 80) if MAX else (g.randint(1, S), g.randint(1, S))
p = g.choice([0.0, 0.2, 0.35])
free = [[g.random() >= p for _ in range(m)] for _ in range(n)]
r, c = g.randrange(n), g.randrange(m)
free[r][c] = True
seen, stack = {(r, c)}, [(r, c)]
while stack:
    x, y = stack.pop()
    for a, b in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
        if 0 <= a < n and 0 <= b < m and free[a][b] and (a, b) not in seen:
            seen.add((a, b)); stack.append((a, b))
print(n, m)
print(r + 1, c + 1, g.choice("NESW"))
for x in range(n):
    print("".join("." if (x, y) in seen else "X" for y in range(m)))
