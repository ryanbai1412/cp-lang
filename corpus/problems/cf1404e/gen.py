#!/usr/bin/env python3
import random
import sys

seed = int(sys.argv[1])
size = int(sys.argv[2])
rng = random.Random(seed)

if size >= 100:
    n = min(200, size)
    m = min(200, size)
    p = 0.7
else:
    n = rng.randint(1, max(1, min(15, size)))
    m = rng.randint(1, max(1, min(15, size)))
    p = rng.uniform(0.2, 0.9)

grid = [["#" if rng.random() < p else "." for _ in range(m)] for _ in range(n)]
if not any(cell == "#" for row in grid for cell in row):
    grid[rng.randrange(n)][rng.randrange(m)] = "#"

print(n, m)
for row in grid:
    print("".join(row))
