"""gen.py SEED SIZE | --max: CCO 2017 P6 (Shifty Grid). An N x M grid, N and M even in [2, 100],
holding 0..NM-1. With both sides even every arrangement is solvable. Small cases are often a
solved grid scrambled by one or two shifts. --max is a random 100 x 100 grid."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
lim = 50 if MAX else max(1, min(50, S))
n = 100 if MAX else 2 * r.randint(1, lim)
m = 100 if MAX else 2 * r.randint(1, lim)
if not MAX and r.random() < 0.3:
    g = [[i * m + j for j in range(m)] for i in range(n)]
    for _ in range(r.randint(0, 2)):
        if r.random() < 0.5:
            i, s = r.randrange(n), r.randrange(m)
            g[i] = g[i][m - s:] + g[i][:m - s]
        else:
            j, s = r.randrange(m), r.randrange(n)
            c = [g[x][j] for x in range(n)]
            for x in range(n):
                g[(x + s) % n][j] = c[x]
    flat = [x for row in g for x in row]
else:
    flat = list(range(n * m))
    r.shuffle(flat)
sys.stdout.write(f"{n} {m}\n" + "".join(" ".join(map(str, flat[i * m:(i + 1) * m])) + "\n" for i in range(n)))
