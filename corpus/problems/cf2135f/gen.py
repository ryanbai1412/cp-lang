import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])

def tree(n, style):
    # grow a full binary tree by splitting leaves; style picks which leaf
    kids = [[0, 0]]
    leaves = [0]
    while len(kids) + 2 <= n:
        if style == 0: i = rng.randrange(len(leaves))
        elif style == 1: i = len(leaves) - 1
        elif style == 2: i = rng.choice([len(leaves) - 1, len(leaves) - 2]) if len(leaves) > 1 else 0
        else: i = rng.randrange(max(1, len(leaves) // 4))
        u = leaves[i]
        a, b = len(kids), len(kids) + 1
        kids += [[0, 0], [0, 0]]
        kids[u] = [a, b]
        if rng.random() < 0.5: leaves[i] = a; leaves.append(b)
        else: leaves[i] = b; leaves.append(a)
    perm = list(range(1, len(kids)))
    rng.shuffle(perm)
    label = [1] + [p + 1 for p in perm]
    out = [None] * len(kids)
    for u, (a, b) in enumerate(kids):
        out[label[u] - 1] = (label[a], label[b]) if a else (0, 0)
    return out

if size > 100:
    tests = [tree(399_999, int(sys.argv[1]) % 4)]
else:
    tests = [tree(2 * rng.randint(0, size) + 1, rng.randrange(4)) for _ in range(rng.randint(1, 4))]
print(len(tests))
for t in tests:
    print(len(t))
    for a, b in t: print(a, b)
