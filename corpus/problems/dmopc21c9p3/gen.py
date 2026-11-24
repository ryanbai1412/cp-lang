import sys, random
# statement: a connected graph, n, m <= 3000, q <= 1e6 queries S T X U V Y.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "3000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 3000
n = 3000 if big else r.randint(2, max(2, min(S, 8)))
m = 3000 if big else r.randint(n - 1, n - 1 + min(n, 4))
edges = [(i, r.randint(max(1, i - 3) if big else 1, i - 1)) for i in range(2, n + 1)]
while len(edges) < m:
    u, v = r.randint(1, n), r.randint(1, n)
    if u != v:
        edges.append((u, v))
q = 10 ** 6 if big else r.randint(1, 8)
print(n, m)
print("\n".join(f"{u} {v}" for u, v in edges))
print(q)
out = []
for _ in range(q):
    s, t, u, v = (r.randint(1, n) for _ in range(4))
    out.append(f"{s} {t} {r.randint(0, n)} {u} {v} {r.randint(0, n)}")
print("\n".join(out))
