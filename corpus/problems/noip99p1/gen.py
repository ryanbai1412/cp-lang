"""noip99p1 generator: whitespace-separated missile heights, each in 0..=50_000."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
hi = rng.choice([5, 50, 50_000])
print(*(rng.randint(0, hi) for _ in range(n)))
