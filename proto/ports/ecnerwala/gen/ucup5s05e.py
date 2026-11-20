import sys, random
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
