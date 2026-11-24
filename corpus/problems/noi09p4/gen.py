import sys, random
if sys.argv[1] == "--max":
    # N = 20, M = 30, every plant protects every plant later in eating order (right to left, row
    # by row), so the protections form a dense DAG (~180k edges, no plant dropped) for the flow.
    r = random.Random(0); n, m = 20, 30
    order = [(i, j) for i in range(n) for j in reversed(range(m))]
    key = {c: k for k, c in enumerate(order)}
    print(n, m)
    for i in range(n):
        for j in range(m):
            later = order[key[(i, j)] + 1:]
            print(r.choice([r.randint(-10**4, 10**4), 10**4, -10**4]), len(later), *[x for c in later for x in c])
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(20, S)); m = r.randint(1, min(30, S))
dense = r.random()
print(n, m)
for i in range(n):
    for j in range(m):
        score = r.choice([r.randint(-10**4, 10**4), r.randint(-20, 20)])
        k = r.randint(0, 3) if r.random() < dense else 0
        cells = [(r.randrange(n), r.randrange(m)) for _ in range(k)]
        print(score, k, *[x for c in cells for x in c])
