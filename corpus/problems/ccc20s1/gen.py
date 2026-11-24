"""gen.py SEED SIZE | --max: CCC 2020 S1. n observations (2 <= n <= 1e5) with distinct times
|t| <= 1e9 and positions |x| <= 1e9. --max is n = 1e5 at the coordinate extremes."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 10 ** 5 if MAX else r.randint(2, max(2, min(10 ** 5, int(sys.argv[2]) + 2)))
span = 10 ** 9 if MAX or r.random() < 0.5 else 20
ts = r.sample(range(-span, span + 1), n) if 2 * span + 1 >= n else r.sample(range(-10 ** 9, 10 ** 9), n)
print(n)
for t in ts: print(t, r.randint(-10 ** 9, 10 ** 9) if MAX else r.randint(-span, span))
