"""noimps23p1 generator: `n m x`, then m tracks `l r` with 1 <= l < r <= n; n, m <= 2e5."""
import random
import sys


def emit(rng, n, m, x, span):
    print(n, m, x)
    for _ in range(m):
        l = rng.randint(1, n - 1)
        print(l, rng.randint(l + 1, min(n, l + span)))


if sys.argv[1] == "--max":
    rng = random.Random(0)
    n = m = 200_000
    print(n, m, n // 2)
    # Short overlapping tracks chain across the whole line, plus long ones to merge over.
    for i in range(m):
        l = i % (n - 1) + 1 if i < n - 1 else rng.randint(1, n - 1)
        print(l, min(n, l + (1 if i < n - 1 else rng.randint(1, n))))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(2, max(2, min(200_000, size)))
emit(rng, n, rng.randint(1, min(200_000, 2 * size)), rng.choice([1, rng.randint(1, n)]), rng.choice([2, n]))
