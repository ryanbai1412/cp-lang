import sys, random
from collections import deque


def case(r, n, m, q, hi):
    order = list(range(1, n + 1))
    r.shuffle(order)
    edges = {(min(order[i], order[p]), max(order[i], order[p]))
             for i in range(1, n) for p in [r.randrange(i)]}
    pairs = n * (n - 1) // 2
    if m >= pairs:
        edges = {(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)}
    while len(edges) < m:
        u, v = r.sample(range(1, n + 1), 2)
        edges.add((min(u, v), max(u, v)))
    adj = [[] for _ in range(n + 1)]
    for u, v in edges:
        adj[u].append(v)
        adj[v].append(u)
    out = [f"{n} {len(edges)} {q}"]
    for u, v in edges:
        out.append(f"{u} {v} {r.randint(1, hi)}" if r.random() < 0.5 else f"{v} {u} {r.randint(1, hi)}")
    dist = {}
    for _ in range(q):
        a, b = r.sample(range(1, n + 1), 2)
        if a not in dist:
            d = [-1] * (n + 1)
            d[a] = 0
            dq = deque([a])
            while dq:
                x = dq.popleft()
                for y in adj[x]:
                    if d[y] < 0:
                        d[y] = d[x] + 1
                        dq.append(y)
            dist[a] = d
        out.append(f"{a} {b} {r.randint(1, dist[a][b])}")
    return out


if sys.argv[1] == "--max":
    r = random.Random(1)
    lines = case(r, 400, 400 * 399 // 2, 300000, 10**9)
    print(1)
    print("\n".join(lines))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 3)
print(t)
for _ in range(t):
    n = r.randint(2, max(2, min(400 // t, 2 + S)))
    m = r.randint(n - 1, n * (n - 1) // 2)
    print("\n".join(case(r, n, m, r.randint(1, 3 * S), r.choice([1, 3, 10, 10**9]))))
