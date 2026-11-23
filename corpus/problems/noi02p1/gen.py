"""noi02p1 generator: t commands `M i j` (i, j in different columns) or `C i j` over n ships (n <= 30000)."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
t = rng.randint(1, max(1, size))
ships = rng.choice([3, 10, 50, 300, 30000] if size < 1000 else [200, 3000, 30000])
col = list(range(ships + 1))
seen = []


def root(x):
    while col[x] != x:
        col[x] = col[col[x]]
        x = col[x]
    return x


def ship():
    if seen and rng.random() < 0.6:
        return rng.choice(seen)
    x = rng.randint(1, ships)
    seen.append(x)
    return x


print(t)
for _ in range(t):
    i, j = ship(), ship()
    if rng.random() < 0.5 and root(i) != root(j):
        col[root(i)] = root(j)
        print("M", i, j)
    else:
        print("C", i, j)
