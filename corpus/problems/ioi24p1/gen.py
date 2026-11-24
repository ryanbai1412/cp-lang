"""ioi24p1 (Nile) inputs: `gen.py SEED SIZE`, or `gen.py --max`, in the sample grader's format.

N, then N lines "W A B" (1 <= W <= 1e9, 1 <= B < A <= 1e9), then Q and Q lines E (1 <= E <= 1e9).
Weights come from a small or a wide range so runs of close weights form at several D. --max:
N = Q = 1e5 with every D from 1 to 1e9 spread on a log scale.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 10**5 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
q = size if MAX else rng.randint(1, size)
wmax = 10**9 if MAX else rng.choice([5, 10 * size, 10**9])
vmax = 10**9 if MAX else rng.choice([3, 100, 10**9])
print(n)
for _ in range(n):
    b = rng.randint(1, vmax - 1) if vmax > 1 else 1
    print(rng.randint(1, wmax), rng.randint(b + 1, vmax), b)
print(q)
for _ in range(q):
    print(min(10**9, max(1, int(10 ** rng.uniform(0, 9)))) if MAX or rng.random() < 0.5 else rng.randint(1, wmax))
