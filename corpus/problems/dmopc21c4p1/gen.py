# Usage: gen.py SEED SIZE. Right triangles with the corner first, (x1, y2) up the vertical leg,
# (x3, y1) along the horizontal leg; queries hit corners, legs, hypotenuses and open space.
# The accepted reference answers 0 for points strictly inside the horizontal leg, so those are
# left to the hand-written tests/10.in (answers from brute.py).
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
B = 10**9
lim = r.choice([5, 20, 1000, B])
def c(): return r.randint(-lim, lim)
def ct(): return r.randint(-lim, lim - 6)  # triangles stay below the fallback queries
def other(v):
    while True:
        w = ct()
        if w != v: return w
n = r.randint(1, min(S, 3000)); q = r.randint(1, min(2 * S, 3000))
tris = []
for _ in range(n):
    x1, y1 = ct(), ct()
    tris.append((x1, y1, x1, other(y1), other(x1), y1))
def on_tri():
    x1, y1, _, y2, x3, _ = r.choice(tris)
    k = r.randint(0, 3)
    if k == 0: return r.choice([(x1, y1), (x1, y2), (x3, y1)])
    if k == 1: return (x1, r.randint(min(y1, y2), max(y1, y2)))
    if k == 2:  # lattice point on the hypotenuse, if any
        from math import gcd
        g = gcd(abs(x3 - x1), abs(y2 - y1)); t = r.randint(0, g)
        return (x1 + (x3 - x1) // g * t, y2 + (y1 - y2) // g * t)
    return (r.randint(min(x1, x3), max(x1, x3)), r.randint(min(y1, y2), max(y1, y2)))
def bad(p):
    return any(p[1] == y1 and min(x1, x3) <= p[0] <= max(x1, x3) and p[0] != x1 for x1, y1, _, _, x3, _ in tris)
def query():
    for _ in range(50):
        p = on_tri() if r.random() < 0.6 else (c(), c())
        if not bad(p): return p
    return (c(), lim - r.randint(0, 5))  # above every triangle's horizontal leg
print(n, q)
for t in tris: print(*t)
for _ in range(q): print(*query())
