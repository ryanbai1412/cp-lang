"""btoi19p2 (Nautilus) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

R, C <= 500 and M <= 5000, an R x C map of '.' and '#', then M signals from NESW?. --max:
R = C = 500, M = 5000 on a mostly-water map with half the signals '?', so the set stays large.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 0 if MAX else int(sys.argv[2])
r = 500 if MAX else rng.randint(1, max(1, min(500, size)))
c = 500 if MAX else rng.randint(1, max(1, min(500, size)))
m = 5000 if MAX else rng.randint(1, max(1, min(5000, 10 * size)))
land = 0.05 if MAX else rng.choice((0.0, 0.1, 0.3, 0.6))
unknown = 0.5 if MAX else rng.choice((0.0, 0.2, 0.7, 1.0))
print(r, c, m)
for _ in range(r):
    print("".join("#" if rng.random() < land else "." for _ in range(c)))
print("".join("?" if rng.random() < unknown else rng.choice("NESW") for _ in range(m)))
