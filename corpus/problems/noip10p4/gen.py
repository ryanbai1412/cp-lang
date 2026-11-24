import sys, random
# seed size: an n x m grid of heights (n, m <= 500, heights <= 1e6); some cases slope down to
# the bottom so every city gets water, the max case is a long descending snake
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 500 if mx else int(sys.argv[2])
n = 500 if mx else r.randint(1, min(500, S + 1)); m = 500 if mx else r.randint(1, min(500, S + 1))
if mx:
    order = [(i, j if i % 2 == 0 else m - 1 - j) for i in range(n) for j in range(m)]
    g = [[0] * m for _ in range(n)]
    for k, (i, j) in enumerate(order): g[i][j] = 1000000 - k
elif r.random() < .5:
    g = [[(n - i) * 1000 + r.randint(0, 999) for j in range(m)] for i in range(n)]
else:
    top = r.choice([3, 10, 1000000])
    g = [[r.randint(0, top) for _ in range(m)] for _ in range(n)]
print(n, m)
for row in g: print(*row)
