#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # n = m = 200, mostly black with scattered holes: about 4e4 flow nodes and 8e4 edges
    rng = random.Random(1)
    n = m = 200
    print(n, m)
    for _ in range(n):
        print("".join("#" if rng.random() < 0.85 else "." for _ in range(m)))
    sys.exit()

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
