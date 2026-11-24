"""yac7p1 inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 1e6).

N (1 <= N <= 1e6), then a_1..a_N with 1 <= a_i <= 1e9; about half the cases are all odd
(subtask 1), the rest mix parities.
--max: N = 1e6 pieces of up to 1e9 pages, mixed parity.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, lim, odd = 10**6, 10**9, False
else:
    size = max(1, int(sys.argv[2]))
    n, lim, odd = rng.randint(1, min(10**6, size)), rng.choice([6, 100, 10**9]), rng.random() < 0.5
a = [rng.randint(1, lim) for _ in range(n)]
if odd:
    a = [x if x % 2 else x - 1 if x > 1 else 1 for x in a]
print(n)
print(*a)
