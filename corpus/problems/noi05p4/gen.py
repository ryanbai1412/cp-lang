import sys, random
if sys.argv[1] == "--max":
    n = e = 1000
    print(n, e); print(1, n)
    for v in range(1, n): print(v, v + 1)
    print(1, n // 2)
    sys.exit()
# Connected simple graph: 2 <= n, n - 1 <= e <= 1000 (statement: N, E <= 1000, no multi-edges).
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, max(2, min(S, 1000)))
e = r.randint(n - 1, min(1000, n * (n - 1) // 2, n - 1 + r.choice([0, 1, 3, n, 2 * n])))
perm = list(range(1, n + 1)); r.shuffle(perm)
edges = set()
path = r.random() < 0.4
for i in range(1, n):
    j = max(0, i - r.randint(1, 3)) if path or r.random() < 0.3 else r.randrange(i)
    edges.add((min(perm[i], perm[j]), max(perm[i], perm[j])))
while len(edges) < e:
    u, v = r.sample(range(1, n + 1), 2)
    edges.add((min(u, v), max(u, v)))
edges = list(edges); r.shuffle(edges)
c = r.randint(1, n); m = c if r.random() < 0.05 else r.randint(1, n)
print(n, len(edges)); print(c, m)
for u, v in edges: print(*((u, v) if r.random() < 0.5 else (v, u)))
