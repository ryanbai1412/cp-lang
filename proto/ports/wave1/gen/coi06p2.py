import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 3); es = set()
for i in range(2, n + 1): es.add((r.randint(1, i - 1), i))
for _ in range(r.randint(0, S)):
    u, v = sorted(r.sample(range(1, n + 1), 2)); es.add((u, v))
es = list(es); r.shuffle(es); print(n, len(es))
for u, v in es: print(*r.sample([u, v], 2))
q = r.randint(1, 2 * S + 1); print(q)
for _ in range(q):
    a, b = r.sample(range(1, n + 1), 2)
    rest = [c for c in range(1, n + 1) if c != a and c != b]
    if rest and r.randint(0, 1): print(2, a, b, r.choice(rest))
    else: print(1, a, b, *r.sample(list(r.choice(es)), 2))
