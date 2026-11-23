"""noi15p4 generator: `n k`, then n weights 1 <= w_i <= 1e11 (one per line); 2 <= k <= 9."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 12
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
k = rng.randint(2, 9)
c = rng.choice([1, 3, 10, 10**6, 10**11])
print(n, k)
for _ in range(n):
    print(rng.randint(1, c))
