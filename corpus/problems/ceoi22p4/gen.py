"""gen.py SEED SIZE | --max: CEOI 2022 Drawing. A tree on N <= 200000 nodes with every degree <= 3,
and N distinct points with integer coordinates in [0, 1e9], no three collinear. Points are
(x, x^2 mod p) for distinct x and a prime p < 1e9, which has no three collinear points even mod p
(sometimes with the axes swapped); for N <= 31622 a quarter of the cases are convex, points on the
parabola y = x^2. --max is N = 200000."""
import random, sys

MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(1, 3 + 2 * S)
P = 999_999_937

deg = [0] * (n + 1)
open_ = [1]
edges = []
shape = r.choice(["random", "path", "caterpillar", "binary"])
for v in range(2, n + 1):
    if shape == "path":
        u = v - 1
    elif shape == "binary":
        u = v // 2
    elif shape == "caterpillar":
        u = v - 1 if v % 2 == 0 or v < 3 else v - 2
        if deg[u] >= 3:
            u = v - 1
    else:
        while True:
            k = r.randrange(len(open_))
            u = open_[k]
            if deg[u] < 3:
                break
            open_[k] = open_[-1]
            open_.pop()
    deg[u] += 1
    deg[v] += 1
    open_.append(v)
    edges.append((u, v))
lab = list(range(1, n + 1))
r.shuffle(lab)
edges = [(lab[u - 1], lab[v - 1]) if r.random() < 0.5 else (lab[v - 1], lab[u - 1]) for u, v in edges]
r.shuffle(edges)

if not MAX and n <= 31622 and r.random() < 0.25:
    xs = r.sample(range(31623), n)
    pts = [(x, x * x) for x in xs]
else:
    xs = r.sample(range(P), n)
    pts = [(x, x * x % P) for x in xs]
    if r.random() < 0.5:
        pts = [(y, x) for x, y in pts]
r.shuffle(pts)
out = [str(n)] + [f"{a} {b}" for a, b in edges] + [f"{x} {y}" for x, y in pts]
sys.stdout.write("\n".join(out) + "\n")
