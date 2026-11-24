"""yac7p6 inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = Q = 5e5).

N Q, then Q operations: "1 L R X V" (add), "2 L R X V" (assign) with 1 <= X < N,
R - L a multiple of X, |V| <= 1e9, and "3 Y". N = 1 allows only queries.
--max: N = Q = 5e5, a third each of adds, assigns and queries; the updates span the whole
array with steps from 1 up past sqrt(N), the case a cell-by-cell solution cannot afford.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n = q = 5 * 10**5
else:
    size = max(1, int(sys.argv[2]))
    n, q = rng.randint(1, min(5 * 10**5, size)), rng.randint(1, min(5 * 10**5, size))
no_assign = not MAX and rng.random() < 0.3
out = [f"{n} {q}"]
for _ in range(q):
    t = rng.randint(1, 3) if n > 1 else 3
    if t == 2 and no_assign:
        t = 1
    if t == 3:
        out.append(f"3 {rng.randint(1, n)}")
        continue
    if MAX:
        x = rng.choice([1, 2, 3, rng.randint(1, 1000), rng.randint(1, n - 1)])
        l = rng.randint(1, x)
        r = l + (n - l) // x * x
    else:
        x = rng.randint(1, min(n - 1, rng.choice([2, 5, n])))
        l = rng.randint(1, n)
        r = l + rng.randint(0, (n - l) // x) * x
    out.append(f"{t} {l} {r} {x} {rng.randint(-10**9, 10**9)}")
print("\n".join(out))
