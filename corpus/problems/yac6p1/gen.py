"""yac6p1 inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 5000, M = 1e5).

N M with 2 <= N <= 5000 and N - 1 <= M <= min(N(N-1)/2, 1e5); about half the cases take
M = N - 1 (subtask 1).
--max: N = 5000, M = 1e5.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, m = 5000, 10**5
else:
    n = rng.randint(2, max(2, min(5000, int(sys.argv[2]))))
    hi = min(n * (n - 1) // 2, 10**5)
    m = n - 1 if rng.random() < 0.5 else rng.randint(n - 1, hi)
print(n, m)
