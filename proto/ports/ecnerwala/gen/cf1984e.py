import sys, random

def tree(r, n):
    shape = r.choice(["random", "path", "star", "caterpillar"])
    edges = []
    for v in range(2, n + 1):
        if shape == "path": p = v - 1
        elif shape == "star": p = 1
        elif shape == "caterpillar": p = max(1, v - r.choice([1, 2]))
        else: p = r.randint(max(1, v - 5), v - 1) if r.random() < 0.5 else r.randint(1, v - 1)
        edges.append((p, v))
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    edges = [(perm[a - 1], perm[b - 1]) for a, b in edges]
    r.shuffle(edges)
    return edges

if sys.argv[1] == "--max":
    r = random.Random(1)
    tests = [200000]
else:
    r = random.Random(int(sys.argv[1]))
    S = int(sys.argv[2])
    tests = [r.randint(2, max(2, S)) for _ in range(r.randint(1, 5))]
print(len(tests))
for n in tests:
    print(n)
    for a, b in tree(r, n):
        print(a, b)
