"""gen.py SEED SIZE | --max: DMOPC '20 Contest 4 P2 (Beautiful Grids). An N x M grid
(1 <= N, M <= 1e18) with K <= min(NM, 3e5) distinct 1-cells. Small cases use small grids,
some huge ones with few cells. --max is K = 3e5 cells on distinct rows and columns of a
1e18 x 1e18 grid, so every row and column flips."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    n, m, k = 10**18, 10**18, 300_000
    rows = r.sample(range(1, 10**18), k)
    cols = r.sample(range(1, 10**18), k)
    cells = list(zip(rows, cols))
else:
    size = int(sys.argv[2])
    big = r.random() < 0.25
    n = r.randint(1, 10**18) if big else r.randint(1, 1 + size)
    m = r.randint(1, 10**18) if big else r.randint(1, 1 + size)
    k = r.randint(0, min(n * m, 300_000, 2 * size + 1))
    cells = set()
    while len(cells) < k:
        cells.add((r.randint(1, n), r.randint(1, m)))
    cells = list(cells)
    r.shuffle(cells)
out = [f"{n} {m} {k}"] + [f"{a} {b}" for a, b in cells]
sys.stdout.write("\n".join(out) + "\n")
