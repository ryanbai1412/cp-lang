"""sac22cc5p2 inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 1e9).

N (1 <= N <= 1e9), then the first extension's direction, left or right.
--max: N = 1e9, left.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 1 if MAX else max(1, int(sys.argv[2]))
print(10**9 if MAX else rng.randint(1, min(10**9, 10 * size)))
print("left" if MAX else rng.choice(["left", "right"]))
