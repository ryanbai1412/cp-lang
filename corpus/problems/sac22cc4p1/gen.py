"""sac22cc4p1 (Obligatory Car Problem) inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 2^31).

One integer N, 0 <= N <= 2^31 (past a 32-bit int).
--max: N = 2^31.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 1 if MAX else max(1, int(sys.argv[2]))
print(2**31 if MAX else rng.choice([rng.randint(0, min(2**31, 4 * size)), rng.randint(0, 2**31)]))
