"""nccc10s4 generator: 3 <= n <= 12 distinct lines, each through two distinct lattice points.

usage: gen.py SEED [MAXN] [C]: n in 3..=MAXN (default 7, small enough for the interpreter),
coordinates in -C..=C (default a small C, so many lines cross at shared points).
"""
import math
import random
import sys

# --max: n = 12 lines through random points with |coordinate| <= 2000 (general position).
if sys.argv[1:] == ["--max"]:
    sys.argv[1:] = ["2022", "12", "2000"]
seed = int(sys.argv[1])
rng = random.Random(seed)
max_n = int(sys.argv[2]) if len(sys.argv) > 2 else 7
c = int(sys.argv[3]) if len(sys.argv) > 3 else rng.choice([2, 3, 5, 2000])
n = 12 if sys.argv[1:] == ["2022", "12", "2000"] else rng.randint(3, max_n)
seen = set()
rows = []
while len(rows) < n:
    x1, y1, x2, y2 = (rng.randint(-c, c) for _ in range(4))
    if (x1, y1) == (x2, y2):
        continue
    a, b, k = y1 - y2, x2 - x1, x1 * y2 - x2 * y1
    g = math.gcd(math.gcd(a, b), k) * (-1 if (a, b) < (0, 0) else 1)
    key = (a // g, b // g, k // g)
    if key in seen:
        continue
    seen.add(key)
    rows.append(f"{x1} {y1} {x2} {y2}")
print(n)
print("\n".join(rows))
