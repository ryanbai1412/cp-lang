"""ioi18p3 generator: `n m q`, m roads of a connected simple graph, then q trips `s e l r`
with l <= s and e <= r."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 200_000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 6)
n = size if big else r.randint(2, max(2, size))
cap = n * (n - 1) // 2
m = 400_000 if big else r.randint(n - 1, min(cap, 3 * n))
q = 200_000 if big else r.randint(1, max(1, size))
roads = set()
for v in range(1, n):
    u = r.randrange(v)
    roads.add((u, v))
while len(roads) < m:
    u, v = r.sample(range(n), 2)
    roads.add((min(u, v), max(u, v)))
roads = list(roads)
r.shuffle(roads)
label = list(range(n))
r.shuffle(label)
print(n, m, q)
for u, v in roads:
    print(label[u], label[v])
for _ in range(q):
    s, e = r.randrange(n), r.randrange(n)
    l = r.randint(0, s)
    rr = r.randint(e, n - 1)
    print(s, e, l, rr)
