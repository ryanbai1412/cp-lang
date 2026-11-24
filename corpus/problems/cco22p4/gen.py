"""CCO 2022 P4 inputs: `gen.py seed size` (a random binary tree of up to `size` nodes rooted at 1,
a balls mostly dropped where their subtree has room), or `gen.py --max` (N = K = 4000: a near-path tree with branches, so the
DP sees deep chains and many two-child nodes; every subtree has room)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 4000 if MAX else int(sys.argv[2])
n = S if MAX else r.randint(1, max(1, S))
label = [1] + r.sample(range(2, n + 1), n - 1)
kids = {u: [0, 0] for u in range(1, n + 1)}
free = [(1, 0), (1, 1)]
for v in label[1:]:
    i = len(free) - 1 - min(len(free) - 1, int(r.expovariate(0.3))) if MAX else r.randrange(len(free))
    free[i], free[-1] = free[-1], free[i]
    u, side = free.pop()
    kids[u][side] = v
    free += [(v, 0), (v, 1)]
par = {kids[u][j]: u for u in kids for j in (0, 1) if kids[u][j]}
def up(v):
    while v in par and r.random() < (0.995 if MAX else 0.6):
        v = par[v]
    return v
if MAX or r.random() < 0.8:
    m = n if MAX else r.randint(0, n)
    balls = [up(v) for v in r.sample(range(1, n + 1), m)]
else:
    m = r.randint(0, n + 1)
    balls = [r.randint(1, n) for _ in range(m)]
print(n, m)
print(*balls)
for u in range(1, n + 1):
    print(*kids[u])
