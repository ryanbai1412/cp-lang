"""ioi12p4 (Ideal City) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, then N distinct blocks "x y" (0 <= x, y < 2^31), connected with connected empty cells. Each
column holds one unbroken run that touches the run before it, which has no holes. --max:
N = 1e5 near 2^31, short columns so the run trees are big and wide.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
base = 2**31 - 2 * 10**5 - 1 if MAX else rng.choice([0, 2**31 - 2 * n - 2])
cells = []
x, lo, hi = base, base + n, base + n
while len(cells) < n:
    tall = rng.randint(1, min(n - len(cells), 5))
    start = rng.randint(lo - tall + 1, hi)
    cells += [(x, start + i) for i in range(tall)]
    lo, hi = start, start + tall - 1
    x += 1
if not MAX and rng.random() < 0.5:
    cells = [(y, x) for x, y in cells]
rng.shuffle(cells)
print(n)
print("\n".join(f"{x} {y}" for x, y in cells))
