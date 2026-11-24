"""NOI 2021 P3 (Celebration): `n m q k` (n, q <= 3e5, m <= 6e5, 0 <= k <= 2), m roads `u v`
making a weakly connected digraph where x => z and y => z imply x => y or y => x, then q
queries `s t` followed by k extra roads `u v` usable only by that query.

Valid graphs come from an out-tree of strongly connected components: each component is a
cycle (or one city), every road goes inside a component or from a component down to one of
its descendants, so every component's predecessors form a chain.

`gen.py SEED SIZE`: n, m and q about SIZE. `gen.py --max`: n = q = 3e5, m = 6e5, k = 2, a
deep tree of small components (long chains to walk), extra roads pointing anywhere.
"""
import random
import sys

big = sys.argv[1:] == ["--max"]
g = random.Random(1 if big else int(sys.argv[1]))
if big:
    n, m, q, k, deep = 300_000, 600_000, 300_000, 2, True
else:
    size = max(1, int(sys.argv[2]))
    n = g.randint(1, size)
    m = g.randint(n - 1, max(n - 1, min(2 * n, size + n)))
    q, k, deep = g.randint(1, size), g.randint(0, 2), g.random() < 0.5

# split the cities into components
perm = list(range(1, n + 1))
g.shuffle(perm)
comps, i = [], 0
while i < n:
    s = g.choice([1, 1, 2, 3]) if deep else g.randint(1, max(1, n // 4))
    comps.append(perm[i:i + s])
    i += s
c = len(comps)
par = [-1] + [max(0, j - g.randint(1, 3)) if deep else g.randrange(j) for j in range(1, c)]
roads = []
for comp in comps:
    if len(comp) > 1:
        roads += [(comp[j], comp[(j + 1) % len(comp)]) for j in range(len(comp))]
for j in range(1, c):
    roads.append((g.choice(comps[par[j]]), g.choice(comps[j])))
# ancestor -> descendant shortcuts, or more roads inside components
anc_depth = 40 if deep else 10
while len(roads) < m:
    j = g.randrange(c)
    a = j
    for _ in range(g.randint(0, anc_depth)):
        if par[a] < 0:
            break
        a = par[a]
    if a == j and len(comps[j]) == 1:
        if c == 1:
            break
        continue
    roads.append((g.choice(comps[a]), g.choice(comps[j])))
g.shuffle(roads)
out = [f"{n} {len(roads)} {q} {k}"] + [f"{u} {v}" for u, v in roads]
for _ in range(q):
    if big and g.random() < 0.7:
        # s a shallow component, t a deep one
        s = g.choice(comps[g.randrange(max(1, c // 10))])
        t = g.choice(comps[c - 1 - g.randrange(max(1, c // 10))])
    else:
        s, t = g.randint(1, n), g.randint(1, n)
    out.append(" ".join(map(str, [s, t] + [g.randint(1, n) for _ in range(2 * k)])))
print("\n".join(out))
