"""noi05p1 generator: an N x M ballroom (N, M <= 200) with the piano on an empty cell, and
K <= 200 consecutive tilt intervals covering 1..=T, T <= 40000."""
import random
import sys


def emit(rng, n, m, k, t, dense):
    grid = [["x" if rng.random() < dense else "." for _ in range(m)] for _ in range(n)]
    x, y = rng.randint(1, n), rng.randint(1, m)
    grid[x - 1][y - 1] = "."
    cuts = sorted(rng.sample(range(1, t), k - 1))
    print(n, m, x, y, k)
    print("\n".join("".join(r) for r in grid))
    for s, e in zip([0] + cuts, cuts + [t]):
        print(s + 1, e, rng.randint(1, 4))


if sys.argv[1] == "--max":
    emit(random.Random(0), 200, 200, 200, 40000, 0.02)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n, m = rng.randint(1, max(1, min(200, size))), rng.randint(1, max(1, min(200, size)))
t = rng.randint(1, max(1, min(40000, 4 * size)))
emit(rng, n, m, rng.randint(1, min(200, t)), t, rng.choice([0.0, 0.1, 0.3]))
