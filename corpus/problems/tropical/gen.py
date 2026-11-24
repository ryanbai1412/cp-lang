"""tropical (Tropical Bananas) inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 2e5, Q = 1e6).

N columns, Q operations "x l r a b" (x in {0, 1}, 1 <= l <= r <= N, |a|, |b| <= 1e5);
small sizes stay in subtask 1 (N, Q <= 2000, |a|, |b| <= 1000).
--max: N = 2e5, Q = 1e6 near-full ranges with extreme a and b, so the totals reach ~2e16.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, q, lim = 2 * 10**5, 10**6, 10**5
else:
    size = max(1, int(sys.argv[2]))
    lim = 1000 if size <= 2000 else 10**5
    n, q = rng.randint(1, min(2 * 10**5, size)), rng.randint(1, min(10**6, size))
out = [f"{n} {q}"]
for _ in range(q):
    if MAX:
        l, r = rng.randint(1, 10), rng.randint(n - 9, n)
        a, b = rng.choice((lim, -lim)), lim
    else:
        l, r = sorted((rng.randint(1, n), rng.randint(1, n)))
        a, b = rng.randint(-lim, lim), rng.randint(-lim, lim)
    out.append(f"{rng.randint(0, 1)} {l} {r} {a} {b}")
print("\n".join(out))
