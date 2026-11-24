"""ioi19p4 (Broken Line): `gen.py SEED SIZE`, or `gen.py --max`.

n, then n lines "x y" (1 <= n <= 1e5, 1 <= x, y <= 1e9, all x distinct and all y
distinct). The output is a broken line from (0, 0) through every dot; checker.py scores it.
--max: n = 1e5 dots of a random permutation spread over the full coordinate range.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 100_000 if MAX else rng.randint(1, min(100_000, max(1, int(sys.argv[2]))))
hi = 1_000_000_000 if MAX else rng.choice([n, 3 * n, 1_000_000_000])
xs = rng.sample(range(1, hi + 1), n)
ys = rng.sample(range(1, hi + 1), n)
if not MAX and rng.random() < 0.3:
    xs.sort()
    ys.sort(reverse=rng.random() < 0.5)
print(n)
print("\n".join(f"{x} {y}" for x, y in zip(xs, ys)))
