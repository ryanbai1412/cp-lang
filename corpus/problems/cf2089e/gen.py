import sys, random
# usage: gen.py seed size; 3 <= n <= 80, t <= 10, sum n <= 80; size >= 80 gives n = 80
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 80:
    ns = [80] if rng.random() < 0.7 else [8] * 10
else:
    ns = []
    while len(ns) < 10 and sum(ns) + 3 <= min(80, 3 * size):
        ns.append(rng.randint(3, min(12, size + 4, 80 - sum(ns))))
        if rng.random() < 0.3: break
print(len(ns))
for n in ns:
    print(n)
    shape = rng.choice(["random", "path", "star", "random"])
    adj = [[] for _ in range(n)]
    for v in range(1, n):
        p = v - 1 if shape == "path" else 0 if shape == "star" else rng.randint(0, v - 1)
        adj[p].append(v)
    order = []
    stack = [0]
    while stack:
        u = stack.pop()
        order.append(u)
        rng.shuffle(adj[u])
        stack.extend(reversed(adj[u]))
    idx = [0] * n
    for pos, u in enumerate(order):
        idx[u] = pos + 1
    edges = []
    for u in range(n):
        for v in adj[u]:
            a, b = idx[u], idx[v]
            edges.append((a, b) if rng.random() < 0.5 else (b, a))
    rng.shuffle(edges)
    for a, b in edges:
        print(a, b)
