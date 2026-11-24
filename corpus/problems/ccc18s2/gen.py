"""gen.py SEED SIZE | --max: CCC 2018 S2. An n x n table (2 <= n <= 100) increasing along rows and
columns, values <= 1e9, rotated by a random multiple of 90 degrees. --max is n = 100."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 100 if MAX else r.randint(2, max(2, min(100, int(sys.argv[2]) + 2)))
step = 10 ** 9 // (2 * n) if MAX or r.random() < 0.5 else 3
g = [[0] * n for _ in range(n)]
for i in range(n):
    for j in range(n):
        lo = max(g[i - 1][j] if i else 0, g[i][j - 1] if j else 0) + 1
        g[i][j] = r.randint(lo, lo + step - 1)
for _ in range(r.randrange(4)):
    g = [[g[n - 1 - j][i] for j in range(n)] for i in range(n)]
print(n)
for row in g: print(*row)
