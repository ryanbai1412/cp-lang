"""noi10p2 generator: `n k L R`, then n notes in -1000..=1000; k is at most the number of chords."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
lo = rng.randint(1, n)
hi = rng.randint(lo, n)
chords = sum(n - m + 1 for m in range(lo, hi + 1))
print(n, rng.randint(1, chords), lo, hi)
print(*(rng.randint(-1000, 1000) for _ in range(n)))
