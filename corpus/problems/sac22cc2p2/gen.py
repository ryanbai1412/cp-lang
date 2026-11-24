"""gen.py SEED SIZE | --max: SAC '22 CC2 P2. An N x N tank (N <= 500) and Q <= 500 queries: `1 x y`
adds a cookie, `2 x1 y1 x2 y2` sprinkles a rectangle. --max is N = Q = 500, cookies first and
then whole-tank rectangles."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 500 if MAX else r.randint(1, min(500, 1 + S))
q = 500 if MAX else r.randint(1, min(500, 1 + 3 * S))
print(n, q)
for i in range(q):
    if (i < q // 2) if MAX else r.random() < 0.5:
        print(1, r.randint(1, n), r.randint(1, n))
    else:
        x1, x2 = (1, n) if MAX else sorted((r.randint(1, n), r.randint(1, n)))
        y1, y2 = (1, n) if MAX else sorted((r.randint(1, n), r.randint(1, n)))
        print(2, x1, y1, x2, y2)
