import random
import sys

if sys.argv[1] == "--max":
    # n = q = 4e5: A toggles edges of one long path, B random edges; many edges live over long
    # stretches of time, so the segment tree pushes every edge into O(log q) nodes
    rng = random.Random(1)
    n = q = 400_000
    order = list(range(1, n + 1))
    rng.shuffle(order)
    pool = [("A", order[i], order[i + 1]) for i in range(0, n - 1, 4)]
    for _ in range(len(pool)):
        x, y = rng.sample(range(1, n + 1), 2)
        pool.append(("B", x, y))
    out = [f"{n} {q}"]
    for _ in range(q):
        g, x, y = rng.choice(pool)
        out.append(f"{g} {x} {y}")
    print("\n".join(out))
    sys.exit()

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
