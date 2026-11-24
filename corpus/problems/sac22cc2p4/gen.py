"""gen.py SEED SIZE | --max: SAC '22 CC2 P4. An N x M grid (N, M <= 1000) of `.` and `C`. Small
sizes vary the cookie density; --max is 1000 x 1000 at density 1/2."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1000 if MAX else r.randint(1, min(1000, 1 + 2 * S))
m = 1000 if MAX else r.randint(1, min(1000, 1 + 2 * S))
p = 0.5 if MAX else r.choice([0.2, 0.5, 0.8])
print(n, m)
for _ in range(n):
    print("".join("C" if r.random() < p else "." for _ in range(m)))
