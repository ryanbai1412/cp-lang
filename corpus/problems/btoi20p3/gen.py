# BOI 2020 Joker: usage `gen.py SEED S`. Graphs are near-bipartite (a random 2-colouring plus a
# few same-colour streets) so odd cycles appear late; no loops or repeated streets.
import sys, random
if sys.argv[1] == "--max":
    # N = M = Q = 2e5: a bipartite graph whose only odd-cycle edges sit in the middle, so limit[]
    # spans the whole range and the divide and conquer does full work; distinct streets, no loops
    r = random.Random(1)
    n = m = q = 200_000
    side = [i % 2 for i in range(n + 1)]
    edges, seen = [], set()
    while len(edges) < m:
        u, v = r.randint(1, n), r.randint(1, n)
        odd = len(edges) in (m // 2 - 3, m // 2, m // 2 + 3)
        if u == v or (side[u] == side[v]) != odd or (min(u, v), max(u, v)) in seen: continue
        seen.add((min(u, v), max(u, v))); edges.append((u, v))
    out = [f"{n} {m} {q}"] + [f"{u} {v}" for u, v in edges]
    for _ in range(q):
        l = r.randint(1, m); out.append(f"{l} {r.randint(l, m)}")
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, max(2, S))
side = [r.randint(0, 1) for _ in range(n + 1)]
cross = r.choice([0.0, 0.02, 0.2, 1.0])
want = r.randint(1, 2 * S)
edges, seen = [], set()
for _ in range(20 * want):
    if len(edges) == want: break
    u, v = r.sample(range(1, n + 1), 2)
    if side[u] == side[v] and r.random() >= cross: continue
    if (min(u, v), max(u, v)) in seen: continue
    seen.add((min(u, v), max(u, v))); edges.append((u, v))
if not edges: edges = [(1, 2)]
m = len(edges); q = r.randint(1, 2 * S)
print(n, m, q)
for u, v in edges: print(u, v)
for _ in range(q):
    l = r.randint(1, m); print(l, r.choice([l, r.randint(l, m), m]))
