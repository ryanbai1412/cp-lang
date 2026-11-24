"""ucrpc21a (Senseless Census) inputs: `gen.py SEED SIZE`, or `gen.py --max` (1000 x 1000).

n m (0 <= n, m <= 1000), then n rows of m characters from "-", "t", "s". An empty grid
is written as n = 0 (no rows).
--max: a 1000 x 1000 grid.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n = m = 1000
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(0, min(1000, size))
    m = rng.randint(1, min(1000, size)) if n else rng.randint(0, min(1000, size))
w = [rng.randint(0, 5) for _ in range(3)] if not MAX else [1, 1, 1]
w = w if sum(w) else [1, 1, 1]
print(n, m)
for _ in range(n):
    print("".join(rng.choices("-ts", weights=w, k=m)))
