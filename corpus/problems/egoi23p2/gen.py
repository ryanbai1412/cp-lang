"""egoi23p2 (Padel Prize Pursuit) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N M", then M matches "x y" (x != y: x beats y and takes y's medals). --max: N = M = 2e5 with
medals passed down long chains (each day's winner beat the day before's), so the merge tree is
as deep as it gets, mixed with random matches.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 2 * 10**5 if MAX else rng.randint(2, max(2, int(sys.argv[2])))
m = 2 * 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
out = [f"{n} {m}"]
last = 0
for i in range(m):
    if MAX and rng.random() < 0.7:
        x = (last + 1) % n
        y = last
    else:
        x, y = rng.sample(range(n), 2)
    last = x
    out.append(f"{x} {y}")
print("\n".join(out))
