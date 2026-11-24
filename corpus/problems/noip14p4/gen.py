"""gen.py SEED SIZE | --max: NOIP 2014 D2T1. d in 1..=20, n in 1..=20 distinct crossings in
0..=128 x 0..=128 with k in 1..=1e6. Small sizes cluster the points so squares overlap; --max is
d = 20, n = 20, k = 1e6."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
d = 20 if MAX else r.randint(1, 20)
n = 20 if MAX else r.randint(1, min(20, 1 + 2 * S))
span = 128 if MAX else r.choice([min(128, 3 + S), 128])
pts = r.sample([(x, y) for x in range(span + 1) for y in range(span + 1)], min(n, (span + 1) ** 2))
print(d)
print(len(pts))
for x, y in pts:
    print(x, y, 10**6 if MAX else r.randint(1, 10**6))
