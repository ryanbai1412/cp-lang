"""ioi14p6io (Holiday, standard I/O) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"n start d" (2 <= n <= 1e5, 0 <= start < n, 0 <= d <= 2n + n / 2), then n
attraction counts 0..=1e9.
--max: n = 1e5, start in the middle third, d = 250000, random counts.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, start, d, top = 100_000, 41_234, 250_000, 1_000_000_000
else:
    size = max(2, int(sys.argv[2]))
    n = rng.randint(2, min(100_000, size))
    start = rng.choice([0, n - 1, rng.randrange(n)])
    d = rng.randint(0, 2 * n + n // 2)
    top = rng.choice([1, 100, 1_000_000_000])
print(n, start, d)
print(" ".join(str(rng.randint(0, top)) for _ in range(n)))
