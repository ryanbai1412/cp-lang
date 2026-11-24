"""apio07p2 (Backup) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

n and k <= n / 2, then n distinct sorted positions in 0..=1e9, one per line. --max: n = 1e5,
k = n / 2 (every office paired) over random gaps, so every split of the cost lists is full.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else rng.randint(2, max(2, int(sys.argv[2])))
k = n // 2 if MAX or rng.random() < 0.3 else rng.randint(1, n // 2)
top = 10**9 if MAX or rng.random() < 0.5 else 3 * n
s = sorted(rng.sample(range(top + 1), n))
print(n, k)
print("\n".join(map(str, s)))
