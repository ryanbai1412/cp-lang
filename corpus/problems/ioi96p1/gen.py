"""ioi96p1 generator: n numbers in 0..=200."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, min(size, 100)))
print(n)
print(*(rng.randint(0, rng.choice([1, 10, 200])) for _ in range(n)))
