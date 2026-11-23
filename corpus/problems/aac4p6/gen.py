"""aac4p6 generator: `n k`, then n values a_i >= 0; k ranges past n to cover k > n."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
k = rng.choice([0, rng.randint(1, n), rng.randint(1, 3 * n), 10**9])
c = rng.choice([1, 3, n, 10**9])
print(n, k)
print(*(rng.randint(0, c) for _ in range(n)))
