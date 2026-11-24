#!/usr/bin/env python3
"""gen.py seed size | --max: OCC '19 G4. N <= 60000 mate pairs, 2N distinct points with
coordinates in 1..=60000. --max: N = 60000 with mates far apart, so every radius is large and
the squares overlap heavily."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 60_000 if MAX else r.randint(1, max(1, int(sys.argv[2])))
C = 60_000 if MAX or r.random() < 0.5 else max(4, 2 * n)
pts = set()
while len(pts) < 2 * n:
    pts.add((r.randint(1, C), r.randint(1, C)))
pts = list(pts)
r.shuffle(pts)
print(n)
print("\n".join(f"{x} {y}" for x, y in pts))
