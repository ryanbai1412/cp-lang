# usage: gen.py SEED [MAX_SIDE]: one network in the sample grader's format:
# `H W K`, then images `r1 c1 r2 c2` (two distinct pixels), then -1.
# 1 <= H, W <= 200, 2 <= H * W, 1 <= K <= H + W - 2.
import random, sys

r = random.Random(int(sys.argv[1]))
side = min(int(sys.argv[2]), 200) if len(sys.argv) > 2 else r.choice([3, 10, 40, 200])
while True:
    shape = r.choice(["row", "col", "square", "wide", "any", "max"])
    if shape == "row":
        h, w = 1, r.randint(2, max(2, side))
    elif shape == "col":
        h, w = r.randint(2, max(2, side)), 1
    elif shape == "square":
        h = w = r.randint(2, max(2, side))
    elif shape == "wide":
        h, w = r.randint(1, max(1, side // 10)), r.randint(1, side)
    elif shape == "max":
        h = w = side
    else:
        h, w = r.randint(1, side), r.randint(1, side)
    if h * w >= 2:
        break
top = h + w - 2
k = r.choice([1, top, r.randint(1, top), max(1, top // 2)])
q = r.randint(4, 60)
cells = [(i, j) for i in range(h) for j in range(w)]
# Images at distance k - 1, k, k + 1 or random, so both answers show up.
print(h, w, k)
for _ in range(q):
    d = r.choice([k - 1, k, k, k + 1, None])
    steps = [(dr, s * (d - abs(dr))) for dr in range(-h + 1, h) for s in (1, -1)
             if d and abs(dr) <= d and d - abs(dr) < w]
    if steps:
        dr, dc = r.choice(steps)
        a = (r.randint(max(0, -dr), min(h - 1, h - 1 - dr)), r.randint(max(0, -dc), min(w - 1, w - 1 - dc)))
        b = (a[0] + dr, a[1] + dc)
    else:
        a, b = r.sample(cells, 2)
    print(*a, *b)
print(-1)
