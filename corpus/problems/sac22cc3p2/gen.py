"""sac22cc3p2 (Unicycle License) inputs: `gen.py SEED SIZE`, or `gen.py --max` (M = 1e9).

One integer M, 1 <= M <= 1e9; small sizes stay near M = 1, where S(t) = 12t^2 - 5t + 1 dips below M.
--max: M = 1e9.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
print(10**9 if MAX else rng.randint(1, min(10**9, max(1, int(sys.argv[2])) ** 3)))
