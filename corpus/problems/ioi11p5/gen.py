"""ioi11p5 (Dancing Elephants): `gen.py SEED SIZE`, or `gen.py --max`.

N L M (1 <= N <= 150000, 0 <= L <= 1e9, M <= 150000 acts), then N sorted positions in
0..1e9, then M lines "i y": elephant i (0 <= i < N) moves to y (0 <= y <= 1e9). Elephants may
share a position. The output is the camera count after each act.
--max: N = M = 150000, positions spread so the answer is in the thousands, every act moving
a random elephant to a random spot.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n = m = 150_000
    hi, l = 1_000_000_000, 200_000
else:
    size = max(1, int(sys.argv[2]))
    n, m = rng.randint(1, min(150_000, size)), rng.randint(1, min(150_000, size))
    hi = rng.choice([10, 100, 1_000_000_000])
    l = rng.randint(0, hi // rng.choice([1, 3, 10]) + 1)
xs = sorted(rng.randint(0, hi) for _ in range(n))
out = [f"{n} {l} {m}", *map(str, xs)]
out += [f"{rng.randrange(n)} {rng.randint(0, hi)}" for _ in range(m)]
print("\n".join(out))
