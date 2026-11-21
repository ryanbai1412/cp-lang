import sys, random

seed, S = (7, 150) if "--max" in sys.argv[1:] else (int(sys.argv[1]), int(sys.argv[2]))
r = random.Random(seed)
full = S >= 8 and seed % 64 == 7
budget = 31 ** 3
cases = []
while len(cases) < 1000:
    if full:
        n, w, k = 31, 6, r.choice([1, 2, 5])
    else:
        n = r.randint(2, min(31, S + 2))
        w = r.randint(1, 8) if r.random() < 0.9 else 100
        a, b = r.randint(0, 4), r.randint(0, 4)
        k = max(1, a * a + b * b) if r.random() < 0.9 else r.randint(1, 20000)
    n = min(n, (w + 1) ** 2 if w < 100 else 10 ** 4)
    if n < 2 or n ** 3 > budget:
        break
    budget -= n ** 3
    x0, y0 = r.randint(1, 100 - min(w, 99)), r.randint(1, 100 - min(w, 99))
    pts = set()
    while len(pts) < n:
        pts.add((min(100, x0 + r.randint(0, w)), min(100, y0 + r.randint(0, w))))
    cases.append((k, list(pts)))
    if full:
        break
print(len(cases))
for k, pts in cases:
    print(len(pts), k)
    for x, y in pts:
        print(x, y)
