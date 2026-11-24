import sys, random
# --max: n = 500, W = 1e6, a real network of 2023 streets with distinct widths (YES).
MAX = sys.argv[1:] == ["--max"]
if MAX: sys.argv[1:] = ["2023", "1000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: 2 <= N <= 500, 1 <= W <= 1e6, 0 <= C, B <= W; group shapes: W = 1, all B equal,
# all C and B equal. Most inputs come from a real street network (YES), some are perturbed or
# random (mostly NO).
n = r.randint(2, min(500, 3 + 2 * S)) if S < 1000 else 500 - r.randrange(3)
W = r.choice([1, r.randint(1, 10), r.randint(1, 10**6), 10**6])
mode = r.choice(["net", "net", "net", "flat", "perturb", "random"])
if MAX: n, W, mode = 500, 10**6, "net"

def matrices(streets):
    # widest bike (max over paths of min b) and widest car (max over paths of min W - b)
    def bottleneck(key):
        comp = [[v] for v in range(n)]; at = list(range(n)); m = [[0] * n for _ in range(n)]
        for u, v, b in sorted(streets, key=key, reverse=True):
            a, c = at[u], at[v]
            if a == c: continue
            if len(comp[a]) < len(comp[c]): a, c = c, a
            for x in comp[a]:
                for y in comp[c]: m[x][y] = m[y][x] = key((u, v, b))
            for y in comp[c]: at[y] = a
            comp[a] += comp[c]; comp[c] = []
        return m
    return bottleneck(lambda s: W - s[2]), bottleneck(lambda s: s[2])

if mode == "flat":
    c, b = r.randint(0, W), r.randint(0, W)
    C = [[c] * n for _ in range(n)]; B = [[b] * n for _ in range(n)]
elif mode == "random":
    C = [[r.randint(0, W) for _ in range(n)] for _ in range(n)]
    B = [[r.randint(0, W) for _ in range(n)] for _ in range(n)]
else:
    m = 2023 if MAX else r.randint(n - 1, min(2023, 3 * n))
    vals = [r.randint(0, W) for _ in range(10**6 if MAX else r.choice([1, 2, 3, 10**6]))]
    perm = list(range(n)); r.shuffle(perm)
    streets = [(perm[i], perm[r.randrange(i)], r.choice(vals)) for i in range(1, n)]
    while len(streets) < m:
        u, v = r.sample(range(n), 2); streets.append((u, v, r.choice(vals)))
    C, B = matrices(streets)
    if mode == "perturb":
        u, v = r.sample(range(n), 2); M = r.choice([C, B])
        M[u][v] = M[v][u] = min(W, max(0, M[u][v] + r.choice([-1, 1])))
print(n, W)
for M in (C, B):
    for j in range(1, n): print(*(M[i][j] for i in range(j)))
