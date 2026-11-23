import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(3, 9)
    pairs = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    shape = random.randint(0, 5)
    if shape == 0:  # star, maybe plus an edge or minus a leaf
        c = random.randint(1, n); es = {tuple(sorted((c, v))) for v in range(1, n + 1) if v != c}
        if random.random() < 0.5: es.discard(random.choice(sorted(es)))
        if random.random() < 0.5: es.add(random.choice(pairs))
    elif shape == 1:  # random tree
        p = list(range(1, n + 1)); random.shuffle(p)
        es = {tuple(sorted((p[i], p[random.randint(0, i - 1)]))) for i in range(1, n)}
    elif shape == 2:  # complete graph or empty graph
        es = set(pairs) if random.random() < 0.5 else set()
    else:  # random density
        q = random.random(); es = {e for e in pairs if random.random() < q}
    es = [e if random.random() < 0.5 else e[::-1] for e in es]; random.shuffle(es)
    print(n, len(es))
    for u, v in es: print(u, v)
