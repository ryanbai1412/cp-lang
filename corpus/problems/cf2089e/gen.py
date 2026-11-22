import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 3)
print(T)
for _ in range(T):
    n = rng.randint(2, min(12, size + 4))
    print(n)
    adj = [[] for _ in range(n)]
    for v in range(1, n):
        p = rng.randint(0, v - 1)
        adj[p].append(v)
    order = []
    def dfs(u):
        order.append(u)
        rng.shuffle(adj[u])
        for v in adj[u]:
            dfs(v)
    dfs(0)
    idx = [0] * n
    for pos, u in enumerate(order):
        idx[u] = pos + 1
    edges = []
    for u in range(n):
        for v in adj[u]:
            a, b = idx[u], idx[v]
            if rng.random() < 0.5:
                edges.append((a, b))
            else:
                edges.append((b, a))
    rng.shuffle(edges)
    for a, b in edges:
        print(a, b)
