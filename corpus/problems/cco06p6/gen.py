"""gen.py SEED SIZE | --max: CCC 2006 Stage 2 P6 Paint by Numbers. An n x m grid (1 <= n, m <= 32)
of random stars, given as its row and column clues, so a fill always exists. Densities vary so
some clues are empty (just 0). --max is a 32 x 32 grid at density one half."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 32 if MAX else r.randint(1, min(32, 1 + S))
m = 32 if MAX else r.randint(1, min(32, 1 + S))
p = 0.5 if MAX else r.choice([0.2, 0.4, 0.5, 0.6, 0.8])
g = ["".join("*" if r.random() < p else "." for _ in range(m)) for _ in range(n)]


def clue(cells):
    return " ".join([str(len(x)) for x in "".join(cells).split(".") if x] + ["0"])


print(n)
print(m)
for row in g:
    print(clue(row))
for j in range(m):
    print(clue(row[j] for row in g))
