import sys, random
# gen.py SEED SIZE: n cities at distinct points of a w x h grid, m portals with random
# rectangles; gen.py --max: n = 7e4, m = 1.5e5, w = h = n
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "70000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 70000:
    n, m, w, h = 70000, 150000, 70000, 70000
else:
    n = r.randint(1, max(1, S))
    w, h = r.randint(1, n), r.randint(1, n)
    while w * h < n:
        w, h = r.randint(1, n), r.randint(1, n)
    m = r.randint(n, 2 * n + 2)
pts = set()
while len(pts) < n:
    pts.add((r.randint(1, w), r.randint(1, h)))
pts = list(pts); r.shuffle(pts)
out = [f"{n} {m} {w} {h}"] + [f"{x} {y}" for x, y in pts]
# city 1 reaches everyone: city i gets a portal from an earlier city covering it
for i in range(1, n):
    x, y = pts[i]
    l, rr = r.randint(1, x), r.randint(x, w)
    d, u = r.randint(1, y), r.randint(y, h)
    if S < 70000 or r.random() < 0.5:
        l, rr, d, u = x, x, y, y
    out.append(f"{r.randint(1, i)} {r.randint(1, 10000)} {l} {rr} {d} {u}")
for _ in range(m - (n - 1)):
    l, rr = sorted(r.randint(1, w) for _ in range(2))
    d, u = sorted(r.randint(1, h) for _ in range(2))
    out.append(f"{r.randint(1, n)} {r.randint(1, 10000)} {l} {rr} {d} {u}")
print("\n".join(out))
