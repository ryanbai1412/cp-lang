"""btoi04p3 (Sequence) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, then N values t_i in 0..=2e9, one per line. --max: N = 1e6 falling values with noise, so
almost every new block merges with the one before (the deepest heap merging).
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**6 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
top = 2 * 10**9 if MAX or rng.random() < 0.5 else 20
if MAX:
    t = [max(0, top - i * 1900 - rng.randint(0, 10**5)) for i in range(n)]
else:
    t = [rng.randint(0, top) for _ in range(n)]
print(n)
print("\n".join(map(str, t)))
