import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size == 8:
    n = q = 400000
    pool = rng.choice([n, 2000])
else:
    n, q = rng.randint(2, 2 * size + 2), rng.randint(1, 6 * size)
    pool = rng.randint(1, 3 * size)
edges = []
for _ in range(pool):
    x, y = rng.sample(range(1, n + 1), 2)
    edges.append((rng.choice("AB"), x, y))
print(n, q)
for _ in range(q):
    g, x, y = rng.choice(edges)
    if rng.random() < 0.5:
        x, y = y, x
    print(g, x, y)
