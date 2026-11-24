"""abc476g (Increasing Popcount) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

T, then T lines "L R" with 1 <= L <= R <= 1e18. --max: T = 1e4, wide ranges near 1e18.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
top = 10**18 if MAX else rng.choice([10, 1000, 10**6, 10**18])
t = 10**4 if MAX else rng.randint(1, max(1, int(sys.argv[2]) // 5))
out = [str(t)]
for i in range(t):
    if MAX and i % 2 == 0:
        l, r = rng.randint(1, 1000), top - rng.randint(0, 1000)
    else:
        l, r = sorted(rng.randint(1, top) for _ in range(2))
    out.append(f"{l} {r}")
print("\n".join(out))
