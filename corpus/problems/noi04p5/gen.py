"""noi04p5 generator: `k1 k2 m n` with slopes 0.1..=50.0 to one decimal and 2 <= m <= n <= 100."""
import random
import sys

if sys.argv[1] == "--max":
    print("49.9 0.1 100 100")
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(2, max(2, min(100, size)))
m = rng.randint(2, n)
k1, k2 = rng.randint(1, 500), rng.randint(1, 500)
print(f"{k1 / 10:.1f} {k2 / 10:.1f} {m} {n}")
