"""ioi09p2 (Hiring) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N W" (1 <= N <= 5e5, 1 <= W <= 1e10), then N candidates "S Q" (1 <= S, Q <= 2e4).
W is at least the smallest S: the accepted C++ always hires someone, so it is wrong when nobody
fits the budget.
--max: N = 5e5 random candidates and W = 1e10, so the sweep keeps and drops many hires.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, w, top = 500_000, 10_000_000_000, 20_000
else:
    size = max(1, int(sys.argv[2]))
    n, top = rng.randint(1, min(500_000, size)), rng.choice([3, 20, 20_000])
ask = [(rng.randint(1, top), rng.randint(1, top)) for _ in range(n)]
if not MAX:
    least = min(s for s, _ in ask)
    w = rng.randint(least, max(least, rng.choice([10, top * top, 10_000_000_000])))
print(n, w)
print("\n".join(f"{s} {q}" for s, q in ask))
