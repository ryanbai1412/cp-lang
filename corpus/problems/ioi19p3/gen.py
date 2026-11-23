"""ioi19p3 generator: an n x m grid in the sample grader's format (`n m`, then the rows).

Mixes small value ranges (many ties and nested valleys), 0/1 grids (subtask 6) and full-range
values; 1 <= n, m <= 2500, 0 <= a <= 7e6.
"""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n, m = rng.randint(1, max(1, size)), rng.randint(1, max(1, size))
kind = rng.choice(["small", "small", "binary", "full"])
hi = {"small": rng.randint(1, 6), "binary": 1, "full": 7_000_000}[kind]
print(n, m)
for _ in range(n):
    print(*(rng.randint(0, hi) for _ in range(m)))
