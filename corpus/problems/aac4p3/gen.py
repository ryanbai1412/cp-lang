import sys, random
# `seed size`: n <= size points (size >= 1e5 gives n = 1e5), distinct x and distinct y, m <= 20
# nonzero slopes. Seeds cycle through random points, all points on one line (5e9 pairs) and
# extreme coordinates/slopes (keys b * y - a * x near 2e18).
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
N = 10 ** 5; C = 10 ** 9
n = N if S >= N else r.randint(2, max(2, S))
mode = int(sys.argv[1]) % 3
m = r.randint(1, 20)
if mode == 0:
    span = C if S >= N else r.choice([n + 2, 3 * n, C])
    xs = r.sample(range(1, span + 1), n); ys = r.sample(range(1, span + 1), n)
    pts = list(zip(xs, ys))
    sl = [(r.choice([-1, 1]) * r.randint(1, 4), r.choice([-1, 1]) * r.randint(1, 4)) for _ in range(m)]
elif mode == 1:
    a, b = r.choice([(1, 1), (1, 2), (3, 1), (-1, 1), (-2, 3)])
    if b < 0: a, b = -a, -b
    step = 1 if S < N else (C - 1) // (n * max(abs(a), b))
    x0 = 1; y0 = 1 if a > 0 else (n - 1) * abs(a) * step + 1
    pts = [(x0 + i * b * step, y0 + i * a * step) for i in range(n)]
    g = r.randint(1, 1000 if S >= N else 3)
    sl = [(a * g * s, b * g * s) for s in (r.choice([-1, 1]) for _ in range(m))]
else:
    xs = r.sample(range(C - 5 * n, C + 1), n); ys = r.sample(range(1, 5 * n + 2), n)
    pts = list(zip(xs, ys))
    sl = [(r.choice([-C, C, -1, 1, C - 1]), r.choice([-C, C, -1, 1, 7])) for _ in range(m)]
r.shuffle(pts)
print(n, m)
for p in pts: print(*p)
for s in sl: print(*s)
