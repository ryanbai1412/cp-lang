import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    n, m = random.randint(2, 6), random.randint(1, 4)
    shape = random.random()
    lab = list(range(1, n + 1)); random.shuffle(lab)
    edges = []
    for v in range(1, n):
        p = v - 1 if shape < 0.3 else 0 if shape < 0.4 else random.randint(0, v - 1)
        e = [lab[p], lab[v]]; random.shuffle(e); edges.append(e)
    random.shuffle(edges)
    pool = list(range(2, n + 1)) if random.random() < 0.7 else [random.randint(2, n)]
    print(n, m)
    for e in edges: print(*e)
    print(*[random.choice(pool) for _ in range(m)])
