import sys, random
# statement: 1 <= n <= 1e5, 0 <= m < n distinct edges forming a forest.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 100000
n = 100000 if big else r.randint(1, max(1, min(S, 100000)))
perm = list(range(1, n + 1))
r.shuffle(perm)
edges = []
path = big or r.random() < 0.3
for i in range(1, n):
    if path and i % 50000 == 0:
        continue
    if not path and r.random() < 0.2:
        continue
    j = i - 1 if path else r.randint(0, i - 1)
    edges.append((perm[i], perm[j]))
r.shuffle(edges)
print(n, len(edges))
for a, b in edges:
    print(a, b)
