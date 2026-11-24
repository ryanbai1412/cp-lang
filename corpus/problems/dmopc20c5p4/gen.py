"""gen.py SEED SIZE | --max: DMOPC '20 Contest 5 P4 (Slacking Off). An N x M grid of Y/B,
2 <= N, M and 4 <= NM <= 2e5, NM capped by SIZE; colours are biased, or rows copied from a few
patterns with sparse flips, so ugly rectangles are common. --max is 447 x 447 all-yellow but
for sparse flips: rows agree almost everywhere, so every row pair sweeps long chunks."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    n = m = 447
    g = [["Y"] * m for _ in range(n)]
    for _ in range(n * m // 200):
        g[r.randrange(n)][r.randrange(m)] = "B"
else:
    cap = max(4, min(200_000, int(sys.argv[2])))
    n = r.randint(2, cap // 2)
    m = r.randint(2, cap // n)
    mode = r.randrange(3)
    if mode == 0:
        p = r.choice([0.5, 0.1, 0.9])
        g = [["Y" if r.random() < p else "B" for _ in range(m)] for _ in range(n)]
    else:
        pats = [[r.choice("YB") for _ in range(m)] for _ in range(r.randint(1, 3))]
        g = [list(r.choice(pats)) for _ in range(n)]
        for _ in range(r.randint(0, n * m // 10)):
            i, j = r.randrange(n), r.randrange(m)
            g[i][j] = "Y" if g[i][j] == "B" else "B"
sys.stdout.write(f"{n} {m}\n" + "".join("".join(row) + "\n" for row in g))
