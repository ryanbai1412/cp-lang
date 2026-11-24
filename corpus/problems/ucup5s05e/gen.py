import sys, random
if sys.argv[1] == "--max":
    # one case with n = q = 1e5 (both sums saturated): the points on two facing parabolas (all on the
    # hull, so both chains are long, floored, so some fall collinear on hull edges), queries mostly
    # strictly inside, some on the hull boundary, at hull vertices and outside.
    r = random.Random(0); n = q = 10**5
    pts = set()
    for i in range(n // 2):
        x = 2 * i - n // 2
        pts.add((x, x * x // 3 - 9 * 10**8)); pts.add((x, 9 * 10**8 - x * x // 3))
    pts = sorted(pts)[:n]
    out = ["1", f"{len(pts)} {q}"] + [f"{x} {y}" for x, y in pts]
    for _ in range(q):
        k = r.random()
        if k < 0.8: a, b = r.choice(pts), r.choice(pts); out.append(f"{(a[0] + b[0]) // 2} {(a[1] + b[1]) // 2}")
        elif k < 0.9: x, y = r.choice(pts); out.append(f"{x} {y}")
        else: out.append(f"{r.randint(-10**9, 10**9)} {r.randint(-10**9, 10**9)}")
    print("\n".join(out)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    w = r.choice([2, 3, 5, 10 * S])
    while True:
        n = r.randint(3, min(3 + 2 * S, (w + 1) ** 2))
        pts = r.sample([(x, y) for x in range(w + 1) for y in range(w + 1)], n)
        (ax, ay), (bx, by) = pts[0], pts[1]
        if any((bx - ax) * (y - ay) - (by - ay) * (x - ax) for x, y in pts): break   # not all on one line
    q = r.randint(1, 6); print(n, q)
    for x, y in pts: print(x, y)
    for _ in range(q): print(r.randint(-1, w + 1), r.randint(-1, w + 1))
