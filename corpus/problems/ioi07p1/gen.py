"""ioi07p1 (Aliens) hidden formations for interactor.py: `gen.py SEED SIZE`, or `gen.py --max`.

"N M BX BY X0 Y0": an N x N meadow (15 <= N <= 2e9), squares of odd side M >= 3, the
5 x 5 board's lower-left cell (BX, BY) with BX + 5M - 1, BY + 5M - 1 <= N, and a
flattened cell (X0, Y0) the program is given.
--max: N = 2e9, M = 399999999 so the board almost fills the meadow, X0, Y0 in the far
corner square.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, m, bx, by, sq = 2_000_000_000, 399_999_999, 3, 2, (4, 4)
else:
    size = max(15, int(sys.argv[2]))
    n = rng.choice([rng.randint(15, max(15, size * size)), rng.randint(15, 2_000_000_000)])
    m = rng.randrange(3, n // 5 + 1, 2) if rng.random() < 0.5 else rng.choice([3, 5, n // 5 - (n // 5 + 1) % 2])
    bx, by = rng.randint(1, n - 5 * m + 1), rng.randint(1, n - 5 * m + 1)
    sq = rng.choice([(i, j) for i in range(5) for j in range(5) if (i + j) % 2 == 0])
x0 = bx + sq[0] * m + rng.randrange(m)
y0 = by + sq[1] * m + rng.randrange(m)
print(n, m, bx, by, x0, y0)
