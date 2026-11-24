"""ioi09p1 (Archery): `gen.py SEED SIZE`, or `gen.py --max`.

N R (1 <= N <= 2e5, 2N <= R <= 1e9), then the 2N ranks, a permutation of 1..2N: yours first,
then the other archers left to right. R is small or near 1e9, your rank anywhere.
--max: N = 2e5, R = 1e9 - 1, a middling rank, the others shuffled.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, r = 200_000, 999_999_999
else:
    n = rng.randint(1, min(200_000, max(1, int(sys.argv[2]))))
    r = rng.choice([2 * n, rng.randint(2 * n, 6 * n + 3), rng.randint(2 * n, 1_000_000_000)])
ranks = list(range(1, 2 * n + 1))
rng.shuffle(ranks)
if MAX:
    mid = ranks.index(n + 1)
    ranks[0], ranks[mid] = ranks[mid], ranks[0]
print(f"{n} {r}")
print("\n".join(map(str, ranks)))
