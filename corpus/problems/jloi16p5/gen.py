"""jloi16p5 (JLOI 2016 XOR Union of Circles) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, then N circles "x y r" (|x|, |y| <= 1e8, 0 < r <= 1e6) no two of which share a point: each pair
is apart or one holds the other. Circles go in nested stacks around a few centres, offset inside
their parent so siblings sit side by side; a small span may hold fewer than N. --max: N = 2e5 in 1e3-deep stacks on a 200-centre grid.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 2 * 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))


def apart(a, b):
    d2 = (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2
    return d2 > (a[2] + b[2]) ** 2 or d2 < (a[2] - b[2]) ** 2


circles = []
if MAX:
    for i in range(200):
        x, y = -10**8 + (i % 20) * 3 * 10**6, -10**8 + (i // 20) * 3 * 10**6
        for k in range(1000):
            circles.append((x + k % 2, y, 10**6 - k * 900))
else:
    span = rng.choice([10, 1000, 10**8])
    rmax = min(10**6, max(1, span // 2))
    tries = 0
    while len(circles) < n and tries < 100 * n:
        tries += 1
        c = (rng.randint(-span, span), rng.randint(-span, span), rng.randint(1, rmax))
        if all(apart(c, o) for o in circles):
            circles.append(c)
rng.shuffle(circles)
print(len(circles))
for c in circles:
    print(*c)
