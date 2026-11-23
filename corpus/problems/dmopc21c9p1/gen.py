"""dmopc21c9p1 generator: n, then the n - 1 neighbouring sums 1 <= a_i <= 1e9."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(2, max(2, min(size, 10**6)))
hi = rng.choice([3, 10, 100, 10**9])
mode = rng.random()
if mode < 0.5:
    # From a real array, so at least one answer exists.
    x = [rng.randint(1, max(1, hi // 2)) for _ in range(n)]
    a = [x[i] + x[i + 1] for i in range(n - 1)]
elif mode < 0.75:
    # Near-tight: x[0] large, the rest small, so the window is narrow.
    x = [rng.randint(1, hi)] + [rng.randint(1, 3) for _ in range(n - 1)]
    a = [min(10**9, x[i] + x[i + 1]) for i in range(n - 1)]
else:
    a = [rng.randint(1, hi) for _ in range(n - 1)]
print(n)
print(*a)
