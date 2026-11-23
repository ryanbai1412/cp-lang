import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(S, 100)); extra = r.randint(0, 3 * n)
cap = r.choice([3, 10, 40, 150, 300])
big = r.randint(0, 5) == 0
edges = []
for v in range(1, n):
    edges.append((r.randrange(v), v, r.randint(1, cap)))
for _ in range(extra):
    edges.append((r.randrange(n), r.randrange(n), r.randint(1, cap)))
if big and edges:
    i = r.randrange(len(edges)); u, v, _ = edges[i]
    edges[i] = (u, v, r.choice([301, 1000, 10**9]))
r.shuffle(edges)
perm = list(range(1, n)); r.shuffle(perm); perm = [0] + perm
print(n, len(edges))
for u, v, w in edges:
    if r.randint(0, 1): u, v = v, u
    print(perm[u], perm[v], w)
