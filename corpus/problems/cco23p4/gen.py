"""cco23p4 (Flip it and Stick it) inputs: `gen.py SEED SIZE`, or `gen.py --max` (|S| = 2e5).

S then T, binary strings, |T| <= 3. --max: T = 000 (the greedy over zero runs) with as many
zeros as stay feasible, in runs of every length.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 2 * 10**5 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
if MAX:
    t = "000"
    runs, used = [], 0
    while used < n:
        z = min(rng.choice([0, 1, 2, 3, 4, 1, 2, 0, 1, 2]), n - used)
        runs.append("0" * z)
        used += z
        if used < n:
            runs.append("1")
            used += 1
    s = "".join(runs)
else:
    t = "".join(rng.choice("01") for _ in range(rng.randint(1, 3)))
    z = rng.random()
    s = "".join("0" if rng.random() < z else "1" for _ in range(n))
print(s)
print(t)
