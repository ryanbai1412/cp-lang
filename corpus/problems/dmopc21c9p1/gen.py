"""dmopc21c9p1 generator: n, then the n - 1 neighbouring sums a_i >= 2."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(2, max(2, size))
if rng.random() < 0.5:
    x = [rng.randint(1, rng.choice([3, 100, 10**9])) for _ in range(n)]
    a = [x[i] + x[i + 1] for i in range(n - 1)]
else:
    a = [rng.randint(2, rng.choice([5, 100, 10**9])) for _ in range(n - 1)]
print(n)
print(*a)
