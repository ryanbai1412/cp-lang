import sys, random
# statement: 1 <= n <= 1e5 nodes and n distinct edges, no self loops, connected.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 100000 if S >= 100000 else r.randint(3, max(3, min(S, 100000)))
perm = list(range(1, n + 1))
r.shuffle(perm)
edges = set()
deep = r.random() < 0.5
for i in range(1, n):
    j = r.randint(max(0, i - 3), i - 1) if deep else r.randint(0, i - 1)
    edges.add((min(perm[i], perm[j]), max(perm[i], perm[j])))
while len(edges) < n:
    a, b = sorted(r.sample(range(1, n + 1), 2))
    edges.add((a, b))
edges = list(edges)
r.shuffle(edges)
print(n)
print("\n".join(f"{a} {b}" if r.random() < 0.5 else f"{b} {a}" for a, b in edges))
