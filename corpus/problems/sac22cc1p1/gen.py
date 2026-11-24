"""gen.py SEED SIZE | --max: SAC '22 CC1 P1. N, M <= 1e5 and M <= C <= 5e5 with C >= N * M (he
always has enough). --max is N * M = C = 5e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
if MAX:
    n, m, c = 5, 100_000, 500_000
else:
    n = r.randint(0, min(100_000, 5 * S))
    m = r.randint(0, min(100_000, 500_000 // max(n, 1), 5 * S))
    c = r.randint(max(m, n * m), min(500_000, n * m + 5 * S + 5))
print(n, m, c, sep="\n")
