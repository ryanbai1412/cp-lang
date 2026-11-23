# BOI 2020 Joker: usage `gen.py SEED S`. Graphs are near-bipartite (a random 2-colouring plus a
# few same-colour streets) so odd cycles appear late; no loops or repeated streets.
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S)
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
