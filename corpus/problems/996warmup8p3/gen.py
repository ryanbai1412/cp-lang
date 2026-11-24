"""996warmup8p3 generator: `n kl kr`, n positions in 0..=L, n directions (1 = right)."""
import random
import sys

L = 10**9 + 1
if sys.argv[1] == "--max":
    rng = random.Random(0)
    n = 10**5
    print(n, 10**9, 10**9 - 1)
    print(*(rng.randint(0, L) for _ in range(n)))
    print(*(rng.randint(0, 1) for _ in range(n)))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
k = rng.choice([3 * n, 10**9])
print(n, rng.randint(1, k), rng.randint(1, k))
print(*(rng.choice([0, L, rng.randint(0, L), rng.randint(0, 9)]) for _ in range(n)))
print(*(rng.randint(0, 1) for _ in range(n)))
