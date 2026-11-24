"""ioi07p2 (Flood) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N points (2 <= N <= 1e5, coordinates 0..=1e6, distinct), then W (1 <= W <= 2N) walls "A B".
Points sit on a lattice and walls join lattice neighbours, so walls are axis-parallel, never
cross and pass through no other point; a lattice has fewer than 2N neighbour pairs.
--max: W = 2N - 633 = 199367, the most walls 1e5 points allow (see port.cp's header): a
317 x 316 lattice less 172 points at the end of its top row, every neighbour pair walled, so
the water takes about 158 hours, a ring of cells an hour.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    k = 317
    xs, ys = [3000 * i for i in range(k)], [3000 * j for j in range(k - 1)]
    present = {(i, j) for i in range(k) for j in range(k - 1) if j < k - 2 or i < k - 172}
else:
    size = max(2, int(sys.argv[2]))
    k = rng.randint(2, max(2, min(316, size)))
    if rng.random() < 0.5:
        xs, ys = sorted(rng.sample(range(1_000_001), k)), sorted(rng.sample(range(1_000_001), k))
    else:
        step = rng.choice([1, 3000])
        xs = ys = [step * i for i in range(k)]
    present = {(i, j) for i in range(k) for j in range(k) if rng.random() < 0.85} | {(0, 0), (1, 0)}


def ring(i, j):
    return min(i, j, k - 1 - i, k - 1 - j)


pairs = [((i, j), (i + di, j + dj)) for (i, j) in present for di, dj in ((1, 0), (0, 1))
         if (i + di, j + dj) in present]
if MAX:
    walls = pairs
else:
    keep = rng.choice([0.3, 0.7, 0.95])
    walls = [(a, b) for a, b in pairs if (a, b) == ((0, 0), (1, 0)) or rng.random() < keep]
pts = sorted(present)
rng.shuffle(pts)
num = {p: t + 1 for t, p in enumerate(pts)}
rng.shuffle(walls)
walls = [(b, a) if rng.random() < 0.5 else (a, b) for a, b in walls]
print(len(pts))
for (i, j) in pts:
    print(xs[i], ys[j])
print(len(walls))
for a, b in walls:
    print(num[a], num[b])
