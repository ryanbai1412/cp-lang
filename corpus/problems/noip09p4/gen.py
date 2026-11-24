import sys, random
MAX = sys.argv[1] == "--max"  # the fewest givens the statement allows (24), so the most search
r = random.Random(0 if MAX else int(sys.argv[1])); S = 57 if MAX else int(sys.argv[2]) if len(sys.argv) > 2 else 57
# A shuffled solved grid, then blanks: at most S of them and at least 24 givens, as the statement asks.
digits = r.sample(range(1, 10), 9)
rows = [b * 3 + i for b in r.sample(range(3), 3) for i in r.sample(range(3), 3)]
cols = [b * 3 + i for b in r.sample(range(3), 3) for i in r.sample(range(3), 3)]
g = [[digits[(3 * (rr % 3) + rr // 3 + cc) % 9] for cc in cols] for rr in rows]
cells = [(i, j) for i in range(9) for j in range(9)]
for i, j in r.sample(cells, 57 if MAX else r.randint(0, min(S, 57))): g[i][j] = 0
kind = 5 if MAX else r.randint(0, 5)
if kind == 0:  # a given changed: often a clash or no completion
    i, j = r.choice([c for c in cells if g[c[0]][c[1]]]); g[i][j] = r.randint(1, 9)
elif kind == 1:  # a blank filled with a random digit
    blanks = [c for c in cells if not g[c[0]][c[1]]]
    if blanks: i, j = r.choice(blanks); g[i][j] = r.randint(1, 9)
for row in g: print(*row)
