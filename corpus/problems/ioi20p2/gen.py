"""ioi20p2 generator: `n`, then a symmetric n x n matrix p with p[i][i] = 1. Most cases are the
path counts of a real layout (rings of trees); some get one entry changed, a ring of two, or a 3."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 1000 if big else r.randint(1, max(1, size))
label = list(range(n))
r.shuffle(label)
p = [[0] * n for _ in range(n)]
v = 0
while v < n:
    comp = r.randint(1, n - v)
    trees = r.randint(1, comp)
    if trees == 2 and not (not big and r.random() < 0.2):
        trees = r.choice([1, 3]) if comp >= 3 else 1
    cut = sorted(r.sample(range(1, comp), trees - 1)) if trees > 1 else []
    bounds = [0] + cut + [comp]
    members = [[v + i for i in range(bounds[k], bounds[k + 1])] for k in range(trees)]
    for k, ts in enumerate(members):
        for l, us in enumerate(members):
            for a in ts:
                for b in us:
                    p[a][b] = 1 if k == l else 2
    v += comp
q = [[p[label[i]][label[j]] for j in range(n)] for i in range(n)]
if not big and n > 1 and r.random() < 0.3:
    i, j = r.sample(range(n), 2)
    q[i][j] = q[j][i] = r.choice([0, 1, 2, 3])
print(n)
for row in q:
    print(*row)
