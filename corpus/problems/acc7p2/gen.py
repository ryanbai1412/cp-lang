"""gen.py SEED SIZE | --max: ACC 7 P2, the interactor's hidden data. Line 1 is N (2 <= N <= 5e3),
line 2 the timestamps t_1..t_N of persons 1..N, a permutation. --max is N = 5e3."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 5_000 if MAX else r.randint(2, min(5_000, 2 + S))
t = list(range(1, n + 1))
r.shuffle(t)
print(n)
print(*t)
