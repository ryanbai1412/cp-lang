"""ioi07p2 (Flood) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N points (2 <= N <= 1e5, coordinates 0..=1e6, distinct), then W (1 <= W <= 2N) walls "A B".
Points sit on a lattice and walls join lattice neighbours, so walls are axis-parallel, never
cross and pass through no other point; a lattice has fewer than 2N neighbour pairs.
--max: a 316 x 316 lattice (99856 points) of 158 nested square rings, each with one gap, and
half the walls between neighbouring rings, so the water takes about 158 hours.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    k = 316
    xs = ys = [3000 * i for i in range(k)]
    present = {(i, j) for i in range(k) for j in range(k)}
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
    gaps = {r: ((r, r + 1), (r, r + 2)) for r in range(k // 2)}
    walls = [(a, b) for a, b in pairs
             if (ring(*a) == ring(*b) and gaps[ring(*a)] != (a, b)) or
             (ring(*a) != ring(*b) and rng.random() < 0.5)]
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
