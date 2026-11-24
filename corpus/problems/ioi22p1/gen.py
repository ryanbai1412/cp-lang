"""ioi22p1 (Catfish Farm): `gen.py SEED SIZE`, or `gen.py --max`.

"N M" then M lines "X Y W" (2 <= N <= 1e5, 1 <= M <= 3e5, distinct cells, 1 <= W <= 1e9);
the output is the largest total weight catchable. --max: N = 1e5, M = 3e5, three fish
in every column at random rows.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n = 100_000
    cells = [(x, y) for x in range(n) for y in rng.sample(range(n), 3)]
    wmax = 1_000_000_000
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(2, max(2, min(100_000, size)))
    rows = rng.randint(1, n)
    m = rng.randint(1, min(300_000, n * rows, size))
    picked = set()
    while len(picked) < m:
        picked.add((rng.randrange(n), rng.randrange(rows)))
    cells = sorted(picked)
    rng.shuffle(cells)
    wmax = rng.choice([10, 1_000_000_000])
print(n, len(cells))
print("\n".join(f"{x} {y} {rng.randint(1, wmax)}" for x, y in cells))
