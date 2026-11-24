"""btoi17p5 (Plus Minus) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, M, K, then K distinct measured cells `sign row col`. Most cases measure a valid grid
(alternating rows, alternating columns or a checkerboard); some flip one cell so the
answer is 0. --max: N = M = 1e9, K = 1e5 cells of an alternating-row grid on 1000 rows.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 10**5 if MAX else max(1, int(sys.argv[2]))
if MAX:
    n = m = 10**9
    k = 10**5
    rows = rng.sample(range(1, n + 1), 1000)
    cells = set()
    while len(cells) < k:
        cells.add((rng.choice(rows), rng.randint(1, m)))
    mode = 0
else:
    side = max(1, min(10**9, size))
    n, m = rng.randint(1, side), rng.randint(1, side)
    k = rng.randint(0, min(n * m, size, 10**5))
    cells = set()
    while len(cells) < k:
        cells.add((rng.randint(1, n), rng.randint(1, m)))
    mode = rng.randrange(3)
start = {}
def sign(r, c):
    if mode == 0:
        return start.setdefault(("r", r), rng.random() < 0.5) != (c % 2 == 1)
    if mode == 1:
        return start.setdefault(("c", c), rng.random() < 0.5) != (r % 2 == 1)
    return start.setdefault("b", rng.random() < 0.5) != ((r + c) % 2 == 1)
cells = sorted(cells)
rng.shuffle(cells)
signs = [sign(r, c) for r, c in cells]
if not MAX and cells and rng.random() < 0.2:
    signs[rng.randrange(len(cells))] ^= True
print(n, m, k)
for (r, c), s in zip(cells, signs):
    print("+" if s else "-", r, c)
