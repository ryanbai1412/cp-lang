"""nccc10s4 generator: n distinct lines, each through two distinct lattice points.

Lines are kept distinct (an assumed statement constraint): on coincident lines
the accepted C++ and the spec disagree, and the statement is not fetched.
"""
import math
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 6
rng = random.Random(seed)
n = rng.randint(1, max(1, min(size, 7)))
c = rng.choice([2, 3, 5])
seen = set()
print(n)
while len(seen) < n:
    x1, y1, x2, y2 = (rng.randint(-c, c) for _ in range(4))
    if (x1, y1) == (x2, y2):
        continue
    a, b, k = y1 - y2, x2 - x1, x1 * y2 - x2 * y1
    g = math.gcd(math.gcd(a, b), k) * (-1 if (a, b) < (0, 0) else 1)
    if (a // g, b // g, k // g) in seen:
        continue
    seen.add((a // g, b // g, k // g))
    print(x1, y1, x2, y2)
