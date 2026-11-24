"""gen.py SEED SIZE | --max: NOIP 2014 D1T1. N, NA, NB in 1..=200 and two gesture cycles over
0..=4. --max is N = NA = NB = 200."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
lim = 200 if MAX else min(200, 1 + 5 * S)
n, na, nb = (200, 200, 200) if MAX else (r.randint(1, lim), r.randint(1, lim), r.randint(1, lim))
print(n, na, nb)
print(*(r.randint(0, 4) for _ in range(na)))
print(*(r.randint(0, 4) for _ in range(nb)))
