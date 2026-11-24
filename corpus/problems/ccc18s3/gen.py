"""gen.py SEED SIZE | --max: CCC 2018 S3. An n x m grid (4 <= n, m <= 100) walled on the border
with '.', 'W', cameras 'C', conveyors LRUD and one start 'S'. --max is 100 x 100."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 100 if MAX else int(sys.argv[2])
n = 100 if MAX else r.randint(4, max(4, min(100, S + 4)))
m = 100 if MAX else r.randint(4, max(4, min(100, S + 4)))
w = r.choice([0.05, 0.15, 0.3]); c = r.choice([0.0, 0.01, 0.03]); belt = r.choice([0.0, 0.1, 0.4])
g = [["W"] * m for _ in range(n)]
for i in range(1, n - 1):
    for j in range(1, m - 1):
        x = r.random()
        g[i][j] = "W" if x < w else "C" if x < w + c else r.choice("LRUD") if x < w + c + belt else "."
g[r.randint(1, n - 2)][r.randint(1, m - 2)] = "S"
print(n, m)
for row in g: print("".join(row))
