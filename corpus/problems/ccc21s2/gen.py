"""gen.py SEED SIZE | --max: CCC 2021 S2. An M x N canvas, M N <= 5e6, and K <= 1e6 row/column strokes.
--max is a 2236 x 2236 canvas with 1e6 strokes."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    n = m = 2236; k = 10**6
else:
    S = int(sys.argv[2])
    n = r.randint(1, S + 1); m = r.randint(1, min(S + 1, 5 * 10**6 // n)); k = r.randint(1, min(10**6, 3 * S))
print(n); print(m); print(k)
print("\n".join(f"R {r.randint(1, n)}" if r.random() < .5 else f"C {r.randint(1, m)}" for _ in range(k)))
