"""gen.py SEED SIZE | --max: CCO 2016 P5. An N x M grid (N, M <= 1e9), K <= 2000 distinct zombie
cells and 0 <= Q <= N + M. --max is a 1e9 x 1e9 grid with 2000 zombies."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
if MAX:
    n = m = 10 ** 9; k = 2000
else:
    n, m = r.randint(1, 3 + 2 * S), r.randint(1, 3 + 2 * S)
    k = r.randint(1, min(n * m, 1 + S))
cells = set()
while len(cells) < k: cells.add((r.randint(1, n), r.randint(1, m)))
q = r.randint(0, 10 ** 6) if MAX else r.randint(0, n + m)
print(n, m); print(k)
for x, y in sorted(cells, key=lambda _: r.random()): print(x, y)
print(q)
