"""gen.py SEED SIZE | --max: ACC 7 P1. T <= 1e4 tests, each 1 <= N <= 1e9. Small cases lean on
N = 1, 2, 3, where the count is not N - 1 for every N. --max is T = 1e4 with large N and the edges."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
t = 10_000 if MAX else r.randint(1, min(10_000, 3 + S))
hi = 10**9 if MAX or r.random() < 0.5 else 5
ns = [r.randint(1, hi) for _ in range(t)]
if MAX:
    ns[:3] = [1, 2, 10**9]
print(t)
print("\n".join(map(str, ns)))
