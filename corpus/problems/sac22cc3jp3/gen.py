"""sac22cc3jp3 (Normal Probabilities) inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 1e5).

N, then N lines each one of A B C D E.
--max: N = 1e5 events, all A but 12 spread B..E, so the product stays well above the 1e-6 tolerance.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else rng.randint(1, max(1, min(10**5, int(sys.argv[2]))))
pool = "AAAAB" if not MAX and rng.random() < 0.5 else "ABCDE"
ps = ["A"] * n if MAX else [rng.choice(pool) for _ in range(n)]
if MAX:
    for k in range(12):
        ps[k * 8000 + 7] = "BCDE"[k % 4]
print(n)
print("\n".join(ps))
