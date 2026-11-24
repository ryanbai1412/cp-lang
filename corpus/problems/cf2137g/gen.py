#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at n = m = q = 2e5 (all three sums): a path 1 -> ... -> n under shuffled labels plus
    # shortcut edges, so painting the sink red cascades class changes up the whole 2e5-deep chain;
    # then paints creep up from the sink, interleaved with asks spread over the path
    r = random.Random(0)
    n = m = q = 200_000
    lab = list(range(1, n + 1)); r.shuffle(lab)
    edges = [(lab[i], lab[i + 1]) for i in range(n - 1)]
    seen = set(edges)
    while len(edges) < m:
        i = r.randrange(n - 2); j = r.randint(i + 2, min(n - 1, i + 5))
        if (lab[i], lab[j]) not in seen:
            seen.add((lab[i], lab[j])); edges.append((lab[i], lab[j]))
    qs = [(1, lab[n - 1])]
    paint = n - 2
    while len(qs) < q:
        if len(qs) % 2 == 0 and paint >= 0:
            qs.append((1, lab[paint])); paint -= 3
        else:
            qs.append((2, lab[r.randrange(n)]))
    out = ["1", f"{n} {m} {q}"] + [f"{u} {v}" for u, v in edges] + [f"{x} {u}" for x, u in qs]
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 30)
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    edges = []
    for i in range(n):
        for j in range(i + 1, n):
            if r.random() < 0.35:
                edges.append((perm[i], perm[j]))
    uncolored = list(range(1, n + 1))
    r.shuffle(uncolored)
    q = r.randint(2, 10 if size == 0 else 40)
    qs = []
    for _ in range(q):
        if uncolored and r.random() < 0.4:
            qs.append((1, uncolored.pop()))
        else:
            qs.append((2, r.randint(1, n)))
    print(n, len(edges), len(qs))
    for u, v in edges:
        print(u, v)
    for op, v in qs:
        print(op, v)
