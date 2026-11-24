import sys, random
# seed size: a connected graph on n <= size + 3 vertices (a tree, a tree plus a few edges, a
# cactus of short cycles, or a dense graph), m <= min(2e5, ...) edges, k in {0, 1}; --max: n = 1e5,
# m = 2e5, k = 1: a cactus of triangles and 4-cycles hung on a random tree, the rest of the edges
# chords inside short windows so blocks of every size show up
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 100000 if mx else r.randint(3, S + 3)
M = 200000 if mx else r.randint(n - 1, min(200000, n - 1 + r.choice([0, 1, 2, 3, S])))
k = 1 if mx else r.randint(0, 1)
kind = "cactus" if mx else r.choice(["tree", "cactus", "dense", "rand"])
p = list(range(n)); r.shuffle(p)
E = set()
def add(u, v):
    if u != v and (u, v) not in E and (v, u) not in E: E.add((u, v))
for i in range(1, n):
    add(p[i], p[r.randint(max(0, i - (3 if kind != "rand" else i)), i - 1)])
if kind == "cactus":
    i = 0
    while i + 3 < n and len(E) + 6 <= M:
        L = r.choice([3, 4, 6])
        for j in range(i, min(n - 1, i + L - 1)): add(p[j], p[j + 1])
        add(p[i], p[min(n - 1, i + L - 1)]); i += L + r.randint(0, 3)
tries = 0
while len(E) < M and tries < 50 * M:
    tries += 1
    i = r.randrange(n)
    w = n if kind == "dense" else (8 if mx else r.choice([2, 4, n]))
    add(p[i], p[min(n - 1, i + r.randint(1, w))])
E = list(E)
r.shuffle(E)
print(n, len(E), k)
for u, v in E: print(u + 1, v + 1)
