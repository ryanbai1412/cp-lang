"""ioi07p5 (Pairs) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"B N D M", then N points in 1..=M per axis, B axes; M <= 7.5e7, 75000, 75 for B = 1, 2, 3,
D < B * M. --max: B = 3, N = 1e5 on the full 75^3 cube with a mid D, so every layer pairs up.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
TOP = {1: 75000000, 2: 75000, 3: 75}
if MAX:
    b, n = 3, 10**5
    m = TOP[b]
    d = 60
else:
    b = rng.randint(1, 3)
    n = rng.randint(1, max(1, int(sys.argv[2])))
    m = rng.randint(1, min(TOP[b], 10 if rng.random() < 0.5 else TOP[b]))
    d = rng.randint(1, b * m - 1) if b * m > 1 else 1
out = [f"{b} {n} {d} {m}"]
for _ in range(n):
    out.append(" ".join(str(rng.randint(1, m)) for _ in range(b)))
print("\n".join(out))
