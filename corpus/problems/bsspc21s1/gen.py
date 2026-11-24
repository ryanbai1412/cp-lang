"""gen.py SEED SIZE | --max: BSSPC '21 S1. An N x M grid (N, M <= 2e3) of lowercase letters and
dots. Cases start from a grid whose rows and columns are palindromes and blank most cells; some
then get one letter changed, which usually makes them impossible. --max is 2000 x 2000, mostly
dots, solvable."""
import random, string, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 2_000 if MAX else r.randint(1, min(2_000, 1 + S))
m = 2_000 if MAX else r.randint(1, min(2_000, 1 + S))
abc = string.ascii_lowercase[: 26 if MAX else r.choice([2, 26])]
g = [[""] * m for _ in range(n)]
for i in range((n + 1) // 2):
    for j in range((m + 1) // 2):
        c = r.choice(abc)
        g[i][j] = g[n - 1 - i][j] = g[i][m - 1 - j] = g[n - 1 - i][m - 1 - j] = c
keep = 0.1 if MAX else r.random()
g = [[c if r.random() < keep else "." for c in row] for row in g]
if not MAX and r.random() < 0.4:
    g[r.randrange(n)][r.randrange(m)] = r.choice(abc)
print(n, m)
print("\n".join("".join(row) for row in g))
