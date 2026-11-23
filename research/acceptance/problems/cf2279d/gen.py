import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.choice([2, 2, 3, random.randint(2, 8), random.randint(5, 8)])
    pairs = [(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)]
    perm = list(range(1, n + 1)); random.shuffle(perm)
    shape = random.random()
    if shape < 0.25: tree = [(perm[i - 1], perm[i]) for i in range(1, n)]
    elif shape < 0.4: tree = [(perm[0], perm[i]) for i in range(1, n)]
    else: tree = [(perm[random.randrange(i)], perm[i]) for i in range(1, n)]
    edges = {tuple(sorted(e)) for e in tree}
    extra = [p for p in pairs if p not in edges]
    k = random.choice([0, 0, random.randint(0, len(extra)), len(extra)])
    edges |= set(random.sample(extra, k))
    edges = list(edges); random.shuffle(edges)
    v = lambda: random.randint(1, n)
    w, x = v(), v()
    y, z = random.choice([(v(), v()), (w, v()), (v(), x), (x, w), (w, x)])
    print(n, len(edges)); print(w, x, y, z)
    for a, b in edges: print(a, b)
