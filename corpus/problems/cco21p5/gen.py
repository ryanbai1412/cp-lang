"""CCO 2021 P5 inputs: `gen.py seed size` (a random road set biased to short roads), or
`gen.py --max` (n = m = 2e5: short roads near each town plus long chords)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(2, max(2, S))
cap = min(n * (n - 1) // 2, 200_000)
m = cap if MAX else r.randint(1, min(cap, max(1, r.choice([n // 2, n, 2 * n, 3 * n]))))
edges = set()
near = 5 if MAX else r.choice([2, 5, n])
while len(edges) < m:
    v = r.randint(2, n)
    u = r.randint(max(1, v - near), v - 1) if r.random() < 0.7 else r.randint(1, n)
    if u != v:
        edges.add((min(u, v), max(u, v)))
print(n, m)
for u, v in r.sample(sorted(edges), m):
    print(*((u, v) if r.random() < 0.5 else (v, u)))
