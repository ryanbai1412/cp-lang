"""noi02p1 generator: t commands `M i j` (i, j in different columns) or `C i j` over ships 1..=30000."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
t = rng.randint(1, max(1, size))
ships = rng.choice([3, 10, 30000]) if size < 1000 else 30000
col = list(range(ships + 1))


def root(x):
    while col[x] != x:
        col[x] = col[col[x]]
        x = col[x]
    return x


print(t)
for _ in range(t):
    i, j = rng.randint(1, ships), rng.randint(1, ships)
    if rng.random() < 0.5 and root(i) != root(j):
        col[root(i)] = root(j)
        print("M", i, j)
    else:
        print("C", i, j)
