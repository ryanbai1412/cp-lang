"""COI 2006 Policija inputs: `gen.py seed size`, or `gen.py --max` (N = 1e5, E = 5e5, Q = 3e5)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 100000 if MAX else r.randint(2, S + 3); es = set()
for i in range(2, n + 1): es.add((r.randint(max(1, i - 3), i - 1) if MAX else r.randint(1, i - 1), i))
while MAX and len(es) < 500000:
    u, v = sorted(r.sample(range(1, n + 1), 2)); es.add((u, v))
for _ in range(0 if MAX else r.randint(0, S)):
    u, v = sorted(r.sample(range(1, n + 1), 2)); es.add((u, v))
es = list(es); r.shuffle(es); print(n, len(es))
for u, v in es: print(*r.sample([u, v], 2))
q = 300000 if MAX else r.randint(1, 2 * S + 1); print(q)
for _ in range(q):
    a, b = r.sample(range(1, n + 1), 2)
    c = r.randint(1, n)
    if n > 2 and r.randint(0, 1):
        while c in (a, b): c = r.randint(1, n)
        print(2, a, b, c)
    else: print(1, a, b, *r.sample(list(r.choice(es)), 2))
