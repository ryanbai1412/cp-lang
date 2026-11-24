import sys, random
if sys.argv[1] == "--max":
    # one case with n = q = 1e5 (both sums): a path on shuffled labels (deepest HLD walks), the
    # slime the m = n/2 vertices at one end, queries spread over the whole path.
    r = random.Random(0); n = q = 10**5; m = n // 2
    lab = list(range(1, n + 1)); r.shuffle(lab)
    out = ["1", f"{n} {m} {q}"] + [f"{lab[i]} {lab[i + 1]}" for i in range(n - 1)]
    slime = lab[:m]; r.shuffle(slime)
    out += [" ".join(map(str, slime)), " ".join(str(r.randint(1, n)) for _ in range(q))]
    print("\n".join(out)); sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = 4
print(t)
for _ in range(t):
    n = r.randint(2, 4 * S + 3)
    m = r.randint(2, n)
    q = r.randint(1, 4 * S + 3)
    print(n, m, q)
    # Generate a random tree
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    adj = {i: [] for i in range(1, n + 1)}
    edges = []
    for i in range(1, n):
        u = perm[i]
        v = perm[r.randint(0, i - 1)]
        if r.random() < 0.5:
            edges.append((u, v))
        else:
            edges.append((v, u))
        adj[u].append(v)
        adj[v].append(u)
    r.shuffle(edges)
    for u, v in edges:
        print(u, v)
    # Generate a connected induced subgraph of size m
    start = r.randint(1, n)
    slime = [start]
    in_slime = {start}
    frontier = list(adj[start])
    while len(slime) < m:
        idx = r.randrange(len(frontier))
        v = frontier.pop(idx)
        if v in in_slime:
            continue
        in_slime.add(v)
        slime.append(v)
        for nb in adj[v]:
            if nb not in in_slime:
                frontier.append(nb)
    r.shuffle(slime)
    print(*slime)
    print(*(r.randint(1, n) for _ in range(q)))
