"""sac22cc3p1 (Hair Hazards) inputs: `gen.py SEED SIZE`, or `gen.py --max` (Q = 99999).

H (100..=1e5), S (1..=1000), Q (1..=floor((H - 1) / S)), one per line.
--max: H = 1e5, S = 1, Q = 99999, the most haircuts to print.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    h, s, q = 10**5, 1, 10**5 - 1
else:
    size = max(1, int(sys.argv[2]))
    h = rng.randint(100, min(10**5, 100 + 10 * size))
    s = rng.randint(1, min(1000, h - 1))
    q = rng.randint(1, (h - 1) // s)
print(h)
print(s)
print(q)
