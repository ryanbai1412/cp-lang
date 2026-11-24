"""noi07p2 (Cash) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N S", then N lines "A_k B_k Rate_k" (0 < A, B <= 10, 0 < Rate <= 100), the best cash staying
within 1e9 (the statement's promise). --max: N = 1e5 days of two slow waves out of phase with
tiny noise, so the hull of earlier days changes every day; prices stay in [1, 9] and S = 1e5
keeps the best trading near 1e7.
"""
import math
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
s = 10**5 if MAX else rng.randint(1, 10**4)
out = [f"{n} {s}"]
for k in range(n):
    if MAX:
        a = 5 + 4 * math.sin(2 * math.pi * k / n) + rng.uniform(-1e-7, 1e-7)
        b = 5 + 4 * math.cos(2 * math.pi * k / n) + rng.uniform(-1e-7, 1e-7)
    else:
        a, b = rng.uniform(4.5, 5.5), rng.uniform(4.5, 5.5)
    out.append(f"{a:.6f} {b:.6f} {rng.uniform(0.01, 100):.6f}")
print("\n".join(out))
