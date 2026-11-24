"""gen.py SEED SIZE | --max: DMOPC '20 Contest 7 P6 (Maou and Division). 3 <= N <= 5e5
distinct points with |x|, |y| <= 1e9: uniform over a box of varying width (narrow boxes force
distance ties), lattice patches, near-collinear runs, or far-apart clusters. --max is
N = 5e5 uniform points over the full coordinate range."""
import random, sys
C = 10**9
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 500000 if MAX else r.randint(3, max(3, min(500000, int(sys.argv[2]))))
mode = "uniform" if MAX else r.choice(["uniform", "narrow", "lattice", "line", "clusters"])
w = C if MAX else r.choice([C, 10**6, 1000, int(n**0.5) + 2])
pts = set()
centres = [(r.randint(-C // 2, C // 2), r.randint(-C // 2, C // 2)) for _ in range(r.randint(2, 6))]
side = int((2 * n) ** 0.5) + 2
step = r.randint(1, max(1, C // (side + 1)))
while len(pts) < n:
    if mode == "uniform":
        p = (r.randint(-w, w), r.randint(-w, w))
    elif mode == "narrow":
        s = max(1, int((4 * n) ** 0.5))
        p = (r.randint(-s, s), r.randint(-s, s))
    elif mode == "lattice":
        p = (r.randint(-side, side) * step, r.randint(-side, side) * step)
        p = (max(-C, min(C, p[0])), max(-C, min(C, p[1])))
    elif mode == "line":
        x = r.randint(-C, C)
        p = (x, max(-C, min(C, x // 3 + r.randint(-2, 2))))
    else:
        cx, cy = r.choice(centres)
        s = max(2, int(n**0.5))
        p = (cx + r.randint(-s, s), cy + r.randint(-s, s))
    pts.add(p)
pts = list(pts)
r.shuffle(pts)
sys.stdout.write(f"{n}\n" + "".join(f"{x} {y}\n" for x, y in pts))
