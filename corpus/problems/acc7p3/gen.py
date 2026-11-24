"""acc7p3 generator: `n m`, n distinct values, then m friendships `u v`."""
import random
import sys

if sys.argv[1] == "--max":
    rng, n, m, cap = random.Random(0), 100000, 100000, 10**9
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
    rng = random.Random(seed)
    n = rng.randint(1, max(1, size))
    m = rng.randint(0, 2 * n)
    cap = rng.choice([3 * n, 10**9])
print(n, m)
print(*rng.sample(range(cap + 1), n))
for _ in range(m):
    print(rng.randint(1, n), rng.randint(1, n))
