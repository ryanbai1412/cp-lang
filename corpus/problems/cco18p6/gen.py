"""cco18p6 (Flop Sorting) inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 4096).

N, then permutations a and b of 1..=N. --max: random permutations, the most merging work.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 4096 if MAX else rng.randint(1, min(4096, max(1, int(sys.argv[2]))))
print(n)
for _ in range(2):
    p = list(range(1, n + 1))
    if MAX or rng.random() < 0.8:
        rng.shuffle(p)
    print(" ".join(map(str, p)))
