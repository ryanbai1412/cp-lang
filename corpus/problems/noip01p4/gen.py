import sys, random
# gen.py SEED SIZE: n cities, each a rectangle (axis-aligned or tilted) given by three corners
# in random order; gen.py --max: n = 100
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = min(100, S) if S >= 100 else r.randint(1, max(1, S))
print(n, r.randint(1, 1000), r.randint(1, n), r.randint(1, n))
for _ in range(n):
    p = (r.randint(0, 500), r.randint(0, 500))
    d = (r.randint(-20, 20), r.randint(-20, 20))
    while d == (0, 0): d = (r.randint(-20, 20), r.randint(-20, 20))
    k = r.randint(1, 3)
    e = (-d[1] * k, d[0] * k)
    pts = [p, (p[0] + d[0], p[1] + d[1]), (p[0] + e[0], p[1] + e[1]), (p[0] + d[0] + e[0], p[1] + d[1] + e[1])]
    r.shuffle(pts)
    print(*[c for q in pts[:3] for c in q], r.randint(1, 1000))
