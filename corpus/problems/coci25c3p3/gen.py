import sys, random
# statement: 1 <= N <= 5e5, N <= M <= 1e6 friendships among 1..=2N, bipartite, with a unique perfect
# matching. Pairs (l_i, r_i) plus edges l_i - r_j only for i < j keep the matching unique.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "500000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 500000
n = 500000 if big else r.randint(1, max(1, min(S, 500000)))
cap = n + n * (n - 1) // 2
m = 1000000 if big else r.randint(n, min(cap, 2 * n + 3, 1000000))
edges = {(i, i) for i in range(n)}
if n > 1 and (big or r.random() < 0.5):
    for i in range(min(n - 1, m - len(edges))):
        edges.add((i, i + 1))
while len(edges) < m:
    i, j = sorted(r.sample(range(n), 2))
    edges.add((i, j))
lab = list(range(1, 2 * n + 1))
r.shuffle(lab)
out = []
for i, j in edges:
    u, v = lab[2 * i], lab[2 * j + 1]
    out.append(f"{u} {v}" if r.random() < 0.5 else f"{v} {u}")
r.shuffle(out)
print(n, m)
print("\n".join(out))
