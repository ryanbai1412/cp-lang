"""abc480f generator: `n m`, then n cake sizes whose sum is divisible by m."""
import random
import sys

if sys.argv[1] == "--max":
    rng, n, m = random.Random(0), 20, 100
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
    rng = random.Random(seed)
    n, m = rng.randint(1, min(20, max(1, size))), rng.randint(1, min(100, max(1, size)))
cap = rng.choice([3, 10, 10**9 - 100]) if sys.argv[1] != "--max" else 10**9 - 100
a = [rng.randint(1, cap) for _ in range(n)]
a[rng.randrange(n)] += (m - sum(a) % m) % m
print(n, m)
print(*a)
