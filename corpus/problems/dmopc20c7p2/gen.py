"""gen.py seed size: DMOPC '20 C7 P2. 1 <= N <= 2e5 tiles at even (x, y), |x|, |y| <= 1e9,
squares where (x + y) % 4 == 0, forming one polygon without holes. Each column of tiles is one
run of y that overlaps the next column's run, so the union is connected and has no holes.
size >= 2e5 gives N near 2e5: by seed, a tall column strip or a wide one, offset near 1e9."""
import random, sys
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 2 * 10 ** 5
target = 2 * 10 ** 5 if big else rng.randint(1, max(1, min(size, 12)))
cols = (1 if seed % 2 else 400) if big else rng.randint(1, max(1, target))
per = max(1, target // cols)
ox = 2 * rng.randint(-10, 10) + (2 * 10 ** 8 * 4 if big else 0)
oy = 2 * rng.randint(-10, 10) - (2 * 10 ** 8 * 4 if big else 0)
tiles, lo, hi = [], 0, per - 1
for c in range(cols):
    if c:
        a = rng.randint(max(0, lo - per), hi)
        lo, hi = a, max(a, min(a + per - 1, hi + per))
        lo = min(lo, hi)
    for y in range(lo, hi + 1):
        if len(tiles) < target: tiles.append((ox + 2 * c, oy + 2 * y))
rng.shuffle(tiles)
print(len(tiles))
for x, y in tiles: print(x, y)
