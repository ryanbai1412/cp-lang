"""sac22cc5p1 (Querying Plants) inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 100).

N, C, then N heights; 1 <= N, H_i, C <= 100.
--max: N = C = 100 and every height 100.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 100 if MAX else rng.randint(1, min(100, max(1, int(sys.argv[2]))))
print(n)
print(100 if MAX else rng.randint(1, 100))
print(*([100] * n if MAX else [rng.randint(1, 100) for _ in range(n)]))
