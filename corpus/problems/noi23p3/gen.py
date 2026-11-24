import sys, random
# seed size: n <= size vertices, a random tree, a path 1-2-..-n (constraint A) or a tree where every
# non-tree edge joins an ancestor and a descendant of 1 (constraint B); m <= size distinct non-tree
# edges, k special vertices; --max: n = m = 5e5, k = n, a random shallow-or-deep tree, half the
# non-tree edges vertical
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 5 * 10**5 if mx else r.randint(2, max(2, S))
shape = "random" if mx else r.choice(["random", "path", "vertical"])
spread = 10 if mx else r.choice([1, 3, n])
par = [0, 0] + [(i - 1 if shape == "path" else r.randint(max(1, i - spread), i - 1)) for i in range(2, n + 1)]
perm = list(range(1, n + 1))
if shape != "path":
    rest = perm[1:]; r.shuffle(rest); perm = [1] + rest
tree = {(min(perm[i - 1], perm[par[i] - 1]), max(perm[i - 1], perm[par[i] - 1])) for i in range(2, n + 1)}
want = 5 * 10**5 if mx else r.randint(1, max(1, min(S, n * (n - 1) // 2 - (n - 1))))
edges = set()
tries = 0
while len(edges) < want and tries < 20 * want + 100:
    tries += 1
    if shape == "vertical" or (mx and r.random() < 0.5):
        b = r.randint(2, n); a = b
        for _ in range(r.randint(1, 30)):
            a = par[a] if par[a] else a
        u, v = perm[a - 1], perm[b - 1]
    else:
        u, v = r.randint(1, n), r.randint(1, n)
    e = (min(u, v), max(u, v))
    if u != v and e not in tree:
        edges.add(e)
edges = list(edges); r.shuffle(edges)
k = n if mx else r.randint(1, n)
print(r.randint(1, 25) if not mx else 25)
print(n, len(edges), k)
for u, v in sorted(tree, key=lambda _: r.random()):
    print(u, v)
for u, v in edges:
    print(u, v)
print(*r.sample(range(1, n + 1), k))
