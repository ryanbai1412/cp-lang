"""ucrpc21b (Lost in the Shuffle) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = 1e4).

n swaps (0 <= n <= 1e4), each "a b" with 1 <= a, b <= 5 (a = b allowed).
--max: n = 1e4 random swaps.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**4 if MAX else rng.randint(0, min(10**4, max(1, int(sys.argv[2]))))
print(n)
for _ in range(n):
    print(rng.randint(1, 5), rng.randint(1, 5))
