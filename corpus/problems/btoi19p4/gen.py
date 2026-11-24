"""btoi19p4 (Tom's Kitchen) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

n, m, k, A[i], B[j] <= 300. --max: n = m = 300, k = 1 and every A, B = 300, so the knapsack
runs over the full sum of B = 90000.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 300 if MAX else max(1, min(300, int(sys.argv[2])))
n = 300 if MAX else rng.randint(1, size)
m = 300 if MAX else rng.randint(1, size)
k = 1 if MAX else rng.randint(1, min(size, rng.choice((1, 2, 3, size))))
top = 300 if MAX else rng.randint(1, size)
a = [300 if MAX else rng.randint(1, top) for _ in range(n)]
b = [300 if MAX else rng.randint(1, top) for _ in range(m)]
print(n, m, k)
print(*a)
print(*b)
