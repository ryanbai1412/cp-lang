"""gen.py SEED SIZE | --max: CCC 2020 S2. An M x N room (1 <= M, N <= 1000) of values 1..=1e6.
--max is 1000 x 1000 with a path along the factor chain."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 1000 if MAX else int(sys.argv[2])
n = 1000 if MAX else r.randint(1, S + 1)
m = 1000 if MAX else r.randint(1, S + 1)
print(n); print(m)
top = n * m if MAX or r.random() < .7 else 10 ** 6
for _ in range(n): print(*[r.randint(1, top) for _ in range(m)])
