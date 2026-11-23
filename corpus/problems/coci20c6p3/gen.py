"""coci20c6p3 generator: `n k`, then n short words over a small alphabet so anagrams repeat."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, min(size, 2000)))
alpha = "abc"[: rng.randint(1, 3)]
print(n, rng.randint(0, min(2000, n * (n - 1) // 2)))
for _ in range(n):
    print("".join(rng.choice(alpha) for _ in range(rng.randint(1, 3))))
