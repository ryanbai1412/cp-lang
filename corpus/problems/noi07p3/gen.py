"""gen.py SEED SIZE | --max: NOI 2007 Surround. "0" (the test number, as in the sample), N M,
then N rows of '.', 'O' and '#'. The statement gives no bounds; the official files reach
3000 x 60 and 100 x 100, keep research cells off the border and always have enough
battalions. A seed grows a few research blobs inside an at most (4 + SIZE)-sided map and
scatters at least as many battalions as the blobs have neighbouring cells, so a wall exists.
--max is 3000 x 60 with blobs in bands and battalions to match."""
import random, sys

if sys.argv[1] == "--max":
    r = random.Random(2007)
    n, m, blobs, extra = 3000, 60, 300, 0.2
else:
    r = random.Random(int(sys.argv[1]))
    s = int(sys.argv[2])
    n, m = r.randint(3, min(3000, 4 + s)), r.randint(3, min(100, 4 + s))
    blobs, extra = r.randint(0, max(1, n * m // 60)), r.choice([0, 0.1, 0.5, 2])
g = [["."] * m for _ in range(n)]
if n > 2 and m > 2:
    for _ in range(blobs):
        x, y = r.randint(1, n - 2), r.randint(1, m - 2)
        for _ in range(r.randint(1, 12)):
            g[x][y] = "O"
            dx, dy = r.choice(((1, 0), (-1, 0), (0, 1), (0, -1)))
            if 1 <= x + dx <= n - 2 and 1 <= y + dy <= m - 2:
                x, y = x + dx, y + dy
around = {(x + dx, y + dy) for x in range(n) for y in range(m) if g[x][y] == "O"
          for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)) if g[x + dx][y + dy] != "O"}
free = [(x, y) for x in range(n) for y in range(m) if g[x][y] == "."]
want = min(len(free), len(around) + int(len(around) * extra) + r.randint(0, 3))
for x, y in r.sample(free, want):
    g[x][y] = "#"
print(0)
print(n, m)
for row in g:
    print("".join(row))
