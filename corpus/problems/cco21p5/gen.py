import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, max(2, S))
cap = min(n * (n - 1) // 2, 200_000)
m = r.randint(0, min(cap, r.choice([n // 2, n, 2 * n, 3 * n])))
edges = set()
near = r.choice([2, 5, n])
while len(edges) < m:
    v = r.randint(2, n)
    u = r.randint(max(1, v - near), v - 1) if r.random() < 0.7 else r.randint(1, n)
    if u != v:
        edges.add((min(u, v), max(u, v)))
print(n, m)
for u, v in r.sample(sorted(edges), m):
    print(*((u, v) if r.random() < 0.5 else (v, u)))
