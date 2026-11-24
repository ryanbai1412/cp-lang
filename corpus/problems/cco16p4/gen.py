"""gen.py seed size: CCO '16 P4. 2 <= N <= 10, 2 <= G <= 1e4 grids of R/W.
Grids are random 2x2 flips of a few base grids, so similar pairs are common;
size >= 1e4 gives N = 10, G = 1e4."""
import random, sys

if sys.argv[1] == "--max":
    # N = 10, G = 1e4 random grids
    rng = random.Random(1)
    print(10); print(10**4)
    print("\n".join("".join(rng.choice("RW") for _ in range(10)) for _ in range(10**5)))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size >= 10 ** 4:
    n, g = 10, 10 ** 4
else:
    n, g = rng.randint(2, min(10, size + 2)), rng.randint(2, size + 2)
bases = [[[rng.randint(0, 1) for _ in range(n)] for _ in range(n)] for _ in range(rng.randint(1, 4))]
print(n); print(g)
for _ in range(g):
    grid = [row[:] for row in rng.choice(bases)]
    if rng.random() < 0.2:
        grid[rng.randrange(n)][rng.randrange(n)] ^= 1
    for _ in range(rng.randint(0, 3 * n)):
        r, c = rng.randrange(n - 1), rng.randrange(n - 1)
        for dr in (0, 1):
            for dc in (0, 1):
                grid[r + dr][c + dc] ^= 1
    for row in grid:
        print("".join("R" if x else "W" for x in row))
