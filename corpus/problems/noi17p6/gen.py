import sys, random
# seed size: n <= size + 3 points (small grids, so collinear points and repeated layers show up,
# or a wide box), m <= size times, k <= min(100, h - 2) for the h vertices of the initial hull,
# which keeps two of them at every time; --max: n = m = 10^5 with 1000 points on a circle of
# radius 10^8 around 99000 in a disk, k = 100 for 10^4 times then 11 (sum 1.99 * 10^6)
def cr(o, a, b): return (a[0]-o[0])*(b[1]-o[1]) - (a[1]-o[1])*(b[0]-o[0])
def hull(q):
    q = sorted(q); lo, up = [], []
    for x in q:
        while len(lo) >= 2 and cr(lo[-2], lo[-1], x) <= 0: lo.pop()
        lo.append(x)
    for x in reversed(q):
        while len(up) >= 2 and cr(up[-2], up[-1], x) <= 0: up.pop()
        up.append(x)
    return lo[:-1] + up[:-1]
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
if mx:
    import math
    n, m, pts = 100000, 100000, set()
    for i in range(1000):
        t = 2 * math.pi * i / 1000
        pts.add((round(1e8 * math.cos(t)), round(1e8 * math.sin(t))))
    while len(pts) < n:
        x, y = r.randint(-9 * 10**7, 9 * 10**7), r.randint(-9 * 10**7, 9 * 10**7)
        if x * x + y * y <= 81 * 10**14: pts.add((x, y))
    pts = list(pts); r.shuffle(pts)
    ks = [100] * 10000 + [11] * 90000
else:
    while True:
        n = r.randint(3, S + 3)
        R = r.choice([1, 2, 3, 5, 10, 10**8])
        if (2 * R + 1) ** 2 < n: continue
        pts = set()
        while len(pts) < n: pts.add((r.randint(-R, R), r.randint(-R, R)))
        pts = list(pts)
        h = len(hull(pts))
        if h >= 3: break
    m = r.randint(1, S)
    ks = [r.randint(0, min(100, h - 2, n)) for _ in range(m)]
print(n, m)
for x, y in pts: print(x, y)
for k in ks: print(k, *[r.randint(0, 2**31 - 1) for _ in range(k)])
