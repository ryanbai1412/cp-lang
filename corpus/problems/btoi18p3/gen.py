"""btoi18p3 (Worm Worries) hidden boxes for interactor.py: `gen.py SEED SIZE`, or `gen.py --max`.

"N M K Q" then the N*M*K humidities 1..=1e9, x-major then y then z. Three kinds as in the
statement: a line (M = K = 1, Q = 35), a sheet (K = 1, N, M <= 1000, Q = 3500) or a small box
(Q = 1e5). --max: a line of N = 1e6.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 0 if MAX else max(1, int(sys.argv[2]))
kind = "line" if MAX else rng.choice(("line", "sheet", "box"))
if kind == "line":
    n, m, k, q = (1_000_000 if MAX else rng.randint(1, min(1_000_000, size * size))), 1, 1, 35
elif kind == "sheet":
    n, m, k, q = rng.randint(1, min(1000, size)), rng.randint(1, min(1000, size)), 1, 3500
else:
    side = max(1, min(20, size))
    n, m, k, q = rng.randint(1, side), rng.randint(1, side), rng.randint(1, side), 100_000
top = rng.choice((3, 1_000_000_000)) if not MAX else 1_000_000_000
print(n, m, k, q)
print(" ".join(str(rng.randint(1, top)) for _ in range(n * m * k)))
