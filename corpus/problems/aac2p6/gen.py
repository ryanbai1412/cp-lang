"""gen.py SEED SIZE | --max: AAC 2 P6. A tree on n <= 2e5 nodes, q <= n days, each a set of X distinct
grounds with the sum of X <= n. Small cases mix paths and random trees. --max is n = 2e5 on a tree with
a 1e5-long spine (deep, so a per-day walk of the tree is slow) and q = 1e5 days of two grounds."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(1, 2 + 2 * S)
spine = n // 2 if MAX else r.randint(1, n)
par = [0, 0] + [i - 1 if i <= spine else r.randint(1, i - 1) for i in range(2, n + 1)]
lab = list(range(1, n + 1))
r.shuffle(lab)
lab = [0] + [1] + [x for x in lab if x != 1]
edges = [(lab[par[i]], lab[i]) for i in range(2, n + 1)]
r.shuffle(edges)
if MAX:
    q, sets = n // 2, [r.sample(range(1, n + 1), 2) for _ in range(n // 2)]
else:
    q, left, sets = r.randint(1, n), n, []
    for d in range(q):
        x = r.randint(1, max(1, left - (q - d - 1)))
        left -= x
        sets.append(r.sample(range(1, n + 1), x))
out = [f"{n} {q}"] + [f"{u} {v}" if r.random() < 0.5 else f"{v} {u}" for u, v in edges]
for s in sets:
    out += [str(len(s)), " ".join(map(str, s))]
sys.stdout.write("\n".join(out) + "\n")
