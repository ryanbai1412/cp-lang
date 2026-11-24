"""gen.py SEED SIZE | --max: CCC 2000 S5. n distinct sheep in the 1000 x 1000 field with two
decimals, sometimes on a coarse grid so x ties and equal distances happen. --max is n = 1000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1000 if MAX else r.randint(1, 2 + 3 * S)
step = 1 if MAX else r.choice([1, 100, 10000, 50000])
n = min(n, (100000 // step + 1) ** 2)
pts = set()
while len(pts) < n:
    pts.add((r.randrange(0, 100001, step), r.randrange(0, 100001, step)))
print(n)
for x, y in pts: print(f"{x / 100:.2f}"); print(f"{y / 100:.2f}")
