"""gen.py SEED SIZE | --max: COCI 2020/2021 Round 6 #4 (Geometrija). 3 <= n <= 1000 distinct
points with |x|, |y| <= 1e9 and no three collinear (rejected exactly). Small cases use a small
grid, some put points in convex position. --max is n = 1000: half on a convex curve at full
range, half random inside."""
import random, sys
from math import gcd
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1000 if MAX else min(1000, r.randint(3, 3 + S))
R = 10**9 if MAX or r.random() < 0.3 else 2 + 3 * min(n, 300)
pts, dirs = [], []
def norm(dx, dy):
    g = gcd(dx, dy)
    dx, dy = dx // g, dy // g
    return (dx, dy) if dx > 0 or (dx == 0 and dy > 0) else (-dx, -dy)
def add(q):
    if q in pts:
        return False
    ds = [norm(q[0] - x, q[1] - y) for x, y in pts]
    if len(set(ds)) < len(ds) or any(d in dirs[i] for i, d in enumerate(ds)):
        return False
    for i, d in enumerate(ds):
        dirs[i].add(d)
    dirs.append(set(ds))
    pts.append(q)
    return True
convex = n // 2 if MAX else (r.randint(0, n) if r.random() < 0.5 else 0)
while len(pts) < convex:
    t = r.randint(-R, R)
    add((t, R - 2 * (t * t // R) if R > 1 else 0))
while len(pts) < n:
    add((r.randint(-R // 2, R // 2), r.randint(-R // 2, R // 2)))
r.shuffle(pts)
sys.stdout.write(f"{n}\n" + "".join(f"{x} {y}\n" for x, y in pts))
