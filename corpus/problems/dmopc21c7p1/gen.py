import sys, random
# statement: an N x M grid, N, M <= 1500, values 0..=1e9 with 0 blank.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1500"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 1500
n = 1500 if big else r.randint(1, max(1, min(S, 6)))
m = 1500 if big else r.randint(1, max(1, min(S, 6)))
step = 400 if big else r.randint(1, 3)
blank = r.choice([0.3, 0.6, 0.9])
spoil = 0 if big else r.random() < 0.3
g = [[(i * m + j) * step + 1 for j in range(m)] for i in range(n)]
for i in range(n):
    for j in range(m):
        if r.random() < blank:
            g[i][j] = 0
        elif spoil and r.random() < 0.2:
            g[i][j] = r.randint(1, (n * m + 1) * step)
print(n, m)
for row in g:
    print(*row)
