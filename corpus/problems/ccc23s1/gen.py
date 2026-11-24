"""gen.py SEED SIZE | --max: CCC 2023 S1. Two rows of c tiles, each 1 (black) or 0 (white).
--max is c = 2e5, mostly black."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
c = 200000 if MAX else r.randint(1, 1 + 4 * S)
p = 0.8 if MAX else r.random()
print(c)
for _ in range(2): print(*[int(r.random() < p) for _ in range(c)])
