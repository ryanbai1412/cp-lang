"""sac22cc5p3 (derangements mod 1e9+7) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

T (1 <= T <= 100), then T values 1 <= N_i <= 1e6, scaled by size.
--max: T = 100, every N_i = 1e6 (no sum bound, so each case pays the full recurrence).
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    ns = [10**6] * 100
else:
    size = max(1, int(sys.argv[2]))
    ns = [rng.randint(1, min(10**6, size)) for _ in range(rng.randint(1, min(100, size)))]
print(len(ns))
print("\n".join(map(str, ns)))
