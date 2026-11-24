"""nccc8s3 generator: `n k`, then n values in 1..=k with every value present."""
import random
import sys
if sys.argv[1] == "--max":
    r = random.Random(0)
    n = k = 200000
    a = list(range(1, k + 1)); r.shuffle(a)
    print(n, k); print(*a)
    sys.exit()

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
k = rng.randint(1, max(1, size))
n = rng.randint(k, max(k, 2 * size))
a = list(range(1, k + 1)) + [rng.randint(1, k) for _ in range(n - k)]
rng.shuffle(a)
print(n, k)
print(*a)
