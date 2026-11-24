"""ioi08p6 (Pyramid Base) inputs: `gen.py SEED SIZE`, `gen.py --max` or `gen.py --max free`.

"M N" (1 <= M, N <= 1e6), B, P, then P obstacles "X1 Y1 X2 Y2 C" (1 <= X1 <= X2 <= M,
1 <= Y1 <= Y2 <= N, 1 <= C <= 7000). Either B = 0 and P <= 400000, or 0 < B <= 2e9 and
P <= 30000.
--max: M = N = 1e6, B = 1e5 and 30000 obstacles costing up to 7000 (the binary search case).
--max free: M = N = 1e6, B = 0 and 400000 small obstacles (the two-pointer case).
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    m = n = 1_000_000
    free = sys.argv[2:] == ["free"]
    budget, p, side = (0, 400_000, 2_000) if free else (100_000, 30_000, 20_000)
    top = 7000
else:
    size = max(1, int(sys.argv[2]))
    m, n = rng.randint(1, min(1_000_000, size)), rng.randint(1, min(1_000_000, size))
    free = rng.random() < 0.5
    p = rng.randint(1, min(30_000, max(1, size)))
    budget = 0 if free else rng.randint(1, rng.choice([10, 7000 * p]))
    side, top = max(1, max(m, n) // rng.choice([1, 2, 4])), rng.choice([3, 7000])
out = [f"{m} {n}", str(budget), str(p)]
for _ in range(p):
    x1, y1 = rng.randint(1, m), rng.randint(1, n)
    x2, y2 = min(m, x1 + rng.randint(0, side - 1)), min(n, y1 + rng.randint(0, side - 1))
    out.append(f"{x1} {y1} {x2} {y2} {rng.randint(1, top)}")
print("\n".join(out))
