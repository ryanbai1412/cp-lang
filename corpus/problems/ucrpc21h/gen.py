import sys, random
if sys.argv[1:] == ["--max"]:
    # n = 100, m = 1e4, every crystal 12 away: total 1188, the loads DP at full size
    r = random.Random(1)
    print(100, 10000)
    for v in range(1, 100): print(0, v, 12)
    for _ in range(10000 - 99): print(r.randrange(1, 100), r.randrange(1, 100), r.randint(12, 300))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(S, 100)); extra = r.randint(0, 3 * n)
cap = r.choice([3, 10, 40, 150, 300])
big = r.randint(0, 5) == 0
edges = []
for v in range(1, n):
    edges.append((r.randrange(v), v, r.randint(1, cap)))
for _ in range(extra):
    edges.append((r.randrange(n), r.randrange(n), r.randint(1, cap)))
if big and edges:
    i = r.randrange(len(edges)); u, v, _ = edges[i]
    edges[i] = (u, v, r.choice([301, 1000, 10**9]))
r.shuffle(edges)
perm = list(range(1, n)); r.shuffle(perm); perm = [0] + perm
print(n, len(edges))
for u, v, w in edges:
    if r.randint(0, 1): u, v = v, u
    print(perm[u], perm[v], w)
