"""ioi06p2 (Pyramid) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"m n a b c d" (3 <= m, n <= 1000; 3 <= a <= m, 3 <= b <= n; 1 <= c <= a - 2, 1 <= d <= b - 2),
then n rows of m elevations in 1..=100.
--max: m = n = 1000, a 500 x 500 base around a 200 x 300 chamber, random elevations.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    m = n = 1000
    a, b, c, d = 500, 500, 200, 300
    top = 100
else:
    size = max(3, int(sys.argv[2]))
    m, n = rng.randint(3, min(1000, size + 2)), rng.randint(3, min(1000, size + 2))
    a, b = rng.randint(3, m), rng.randint(3, n)
    c, d = rng.randint(1, a - 2), rng.randint(1, b - 2)
    top = rng.choice([3, 100])
print(m, n, a, b, c, d)
for _ in range(n):
    print(*(rng.randint(1, top) for _ in range(m)))
