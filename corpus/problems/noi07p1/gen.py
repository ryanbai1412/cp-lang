# Usage: gen.py SEED [MAXN]. A connected simple graph within the statement
# (n <= 100, m <= 4500, 1 <= c <= 1000) whose shortest-path counts stay <= 1e10.
# Small weight sets and grid-like layers make many tied shortest paths.
import random
import sys
if sys.argv[1] == "--max":
    r = random.Random(0)
    n, m = 100, 4500
    es = [(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)]
    r.shuffle(es)
    print(n, m)
    for a, b in es[:m]: print(a, b, 1)
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2]) if len(sys.argv) > 2 else 100


def counts(n, edges):
    INF = float("inf")
    d = [[INF] * n for _ in range(n)]
    w = [[0] * n for _ in range(n)]
    for i in range(n):
        d[i][i], w[i][i] = 0, 1
    for a, b, c in edges:
        d[a][b] = d[b][a] = c
        w[a][b] = w[b][a] = 1
    for k in range(n):
        for i in range(n):
            if i == k or d[i][k] == INF:
                continue
            for j in range(n):
                if j == k or j == i:
                    continue
                c = d[i][k] + d[k][j]
                if c < d[i][j]:
                    d[i][j], w[i][j] = c, w[i][k] * w[k][j]
                elif c == d[i][j]:
                    w[i][j] += w[i][k] * w[k][j]
    return max(max(row) for row in w)


def make():
    n = r.randint(2, max(2, S))
    shape = r.choice(["tree", "sparse", "dense", "layers"])
    wts = r.choice([[1], [1, 2], [1, 2, 3], list(range(1, 1001))])
    perm = list(range(n))
    r.shuffle(perm)
    es = set()
    if shape == "layers":
        width = r.randint(1, 4)
        for i in range(n):
            for j in range(i + 1, min(n, (i // width + 2) * width)):
                if j // width == i // width + 1 or r.random() < 0.05:
                    es.add((i, j))
        for i in range(1, n):
            if not any((a, i) in es for a in range(i)):
                es.add((i - 1, i))
    else:
        for i in range(1, n):
            es.add((r.randrange(i), i))
        cap = min(4500, n * (n - 1) // 2)
        extra = {"tree": 0, "sparse": r.randint(0, 2 * n), "dense": cap}[shape]
        target = min(cap, len(es) + extra)
        while len(es) < target:
            a, b = r.sample(range(n), 2)
            es.add((min(a, b), max(a, b)))
    edges = [(perm[a], perm[b], r.choice(wts)) for a, b in es]
    r.shuffle(edges)
    return n, edges


while True:
    n, edges = make()
    if counts(n, edges) <= 10**10:
        break
print(n, len(edges))
for a, b, c in edges:
    print(a + 1, b + 1, c) if r.random() < 0.5 else print(b + 1, a + 1, c)
