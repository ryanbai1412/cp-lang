"""abc481g generator: n, then n pairs `a b` with 1 <= a < b < 998244353."""
import random
import sys

if sys.argv[1] == "--max":
    rng, n, cap = random.Random(0), 120000, 998244352
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
    rng = random.Random(seed)
    n, cap = rng.randint(1, min(10, max(1, size))), rng.choice([4, 20, 998244352])
print(n)
for _ in range(n):
    a = rng.randint(1, cap - 1)
    print(a, rng.randint(a + 1, cap))
