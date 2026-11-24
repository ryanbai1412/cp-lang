"""ucrpc21i (Sphere Mongers) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = 1000).

n r, then n spheres "x y g" with 0 < n <= 1000, 0 <= x, y <= 1000, g in {0, 1}. The
statement gives no bound on r; inputs keep 1 <= r <= 1000, the coordinate span.
Small sizes pack the spheres into a small box so circles meet and points coincide.
--max: n = 1000 in a 60 x 60 box with r = 20, so most of the ~5e5 pairs give candidates.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, r, box = 1000, 20, 60
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(1, min(1000, size))
    box = rng.choice([3, 10, 100, 1000])
    r = rng.randint(1, rng.choice([3, 10, 1000]))
print(n, r)
for _ in range(n):
    print(rng.randint(0, box), rng.randint(0, box), rng.randint(0, 1))
