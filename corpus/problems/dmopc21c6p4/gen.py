import sys, random
# statement: a connected simple graph, 4 <= n, m <= 2e5, distinct houses A, B, C, D.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 200000
n = 200000 if big else r.randint(4, max(4, min(S, 9)))
extra = (200000 - (n - 1)) if big else r.randint(0, min(n * (n - 1) // 2 - (n - 1), 2 * n))
order = list(range(1, n + 1)); r.shuffle(order)
edges = set()
for i in range(1, n):
    u, v = order[i], order[r.randrange(max(0, i - 3) if big and S % 2 else 0, i)]
    edges.add((min(u, v), max(u, v)))
while len(edges) < n - 1 + extra:
    u, v = r.sample(range(1, n + 1), 2)
    edges.add((min(u, v), max(u, v)))
edges = list(edges); r.shuffle(edges)
print(n, len(edges))
for u, v in edges:
    print(u, v) if r.random() < 0.5 else print(v, u)
print(*r.sample(range(1, n + 1), 4))
