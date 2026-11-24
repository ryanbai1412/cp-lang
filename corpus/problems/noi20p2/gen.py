"""noi20p2 (Destiny) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = m = 5e5).

n, the n - 1 edges, m, then m pairs (u, v) with u a proper ancestor of v (rooted at 1).
--max: a caterpillar, a spine of 4e5 with leaves hung on it, so depths reach 4e5 and every
pair spans a long stretch of the spine.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 5 * 10**5 if MAX else max(2, int(sys.argv[2]))
n = size if MAX else rng.randint(2, size)
m = size if MAX else rng.randint(1, size)
par = [-1] * n
if MAX:
    spine = 4 * 10**5
    for i in range(1, n):
        par[i] = i - 1 if i < spine else rng.randrange(spine)
else:
    kind = rng.randrange(3)
    for i in range(1, n):
        par[i] = i - 1 if kind == 0 else rng.randrange(i) if kind == 1 else rng.randrange(max(0, i - 3), i)
lab = [0] + rng.sample(range(1, n), n - 1)  # vertex 0 is the root, label 1
pairs = []
for _ in range(m):
    v = rng.randrange(1, n)
    if MAX:
        top = v if v < spine else par[v] + 1  # spine 0..top-1 are v's proper ancestors
        u = rng.randrange(top) if rng.random() < 0.5 else rng.randrange(max(0, top - 1000), top)
    else:
        anc, x = [], par[v]
        while x >= 0:
            anc.append(x)
            x = par[x]
        u = rng.choice(anc)
    pairs.append((lab[u] + 1, lab[v] + 1))
es = [(lab[par[i]] + 1, lab[i] + 1) for i in range(1, n)]
rng.shuffle(es)
out = [str(n)] + [f"{u} {v}" for u, v in es] + [str(m)] + [f"{u} {v}" for u, v in pairs]
print("\n".join(out))
