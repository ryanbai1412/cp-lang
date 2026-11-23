"""ucrpc21c generator: health n, then m rounds of damage `u v x y` (each team takes u*v and x*y)."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
m = rng.randint(1, max(1, size))
c = rng.choice([1, 3, 10])
n = rng.randint(1, rng.choice([5, 10 * m, 100 * m * c]))
print(n, m)
for _ in range(m):
    print(*(rng.randint(0, c) for _ in range(4)))
