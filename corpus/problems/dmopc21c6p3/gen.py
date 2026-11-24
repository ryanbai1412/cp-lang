import sys, random
# statement: an N x M grid, N, M <= 1500, K <= 3000 (n + m), colours 0..=1e9, 0 blank.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1500"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 1500
n = 1500 if big else r.randint(1, max(1, min(S, 7)))
m = 1500 if big else r.randint(1, max(1, min(S, 7)))
k = 3000 if big else r.randint(0, n + m)
dots = 40 if big else r.randint(0, n * m)
top = 10 ** 9 if big or r.random() < 0.3 else 3
g = [[0] * m for _ in range(n)]
for _ in range(dots):
    g[r.randrange(n)][r.randrange(m)] = r.randint(1, top)
print(n, m, k)
for row in g:
    print(*row)
