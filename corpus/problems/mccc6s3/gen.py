"""mccc6s3 (Optimal Split Point) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = 1e6).

n then a_1..a_n in 1..=1e6. --max: values up to 1e6 with a heavy tail, so the side sums keep
crossing and the nearest-value searches skip many used values.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 10**6 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
hi = 10**6 if MAX else rng.choice([1, 3, 20, 10**6])
if MAX:
    a = [rng.randint(1, hi) if rng.random() < 0.5 else rng.randint(1, 1000) for _ in range(n)]
else:
    a = [rng.randint(1, hi) for _ in range(n)]
print(n)
print(" ".join(map(str, a)))
