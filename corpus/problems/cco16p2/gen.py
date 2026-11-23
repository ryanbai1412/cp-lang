import sys, random
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
C, W = 10**6, 10**4


def distinct(n, draw):
    pts = set()
    while len(pts) < n:
        pts.add(draw())
    return list(pts)


if size <= 20:
    n = rng.randint(2, size + 2)
    r = rng.choice([1, 2, 4, 50])
    r = max(r, 2)
    pts = distinct(n, lambda: (rng.randint(-r, r), rng.randint(-r, r)))
    wr = rng.choice([3, W])
else:
    n = 4000
    mode = seed % 5
    if mode == 0:  # general position in the full box
        pts = distinct(n, lambda: (rng.randint(-C, C), rng.randint(-C, C)))
    elif mode == 1:  # a 64 x 63 grid scaled to the corners: heavy collinearity
        pts = [(-C + 31_000 * i, -C + 31_000 * j) for i in range(64) for j in range(63)][:n]
    elif mode == 2:  # everyone on one line
        pts = [(-C + 500 * i, -C + 500 * i) for i in range(n)]
    elif mode == 3:  # a few lines through the origin
        pts = distinct(n, lambda: (lambda t, d: (t * d[0], t * d[1]))(
            rng.randint(-C // 7, C // 7), rng.choice([(1, 0), (0, 1), (1, 1), (3, -7), (-5, 2)])))
    else:  # small box, lots of collinear triples
        pts = distinct(n, lambda: (rng.randint(-40, 40), rng.randint(-40, 40)))
    wr = W
print(n)
for x, y in pts:
    print(x, y, rng.choice([-wr, wr]) if rng.random() < 0.2 else rng.randint(-wr, wr))
