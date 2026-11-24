"""cco05p4 (Primed Sequences) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

t <= 20 tests, each `n a_1 .. a_n` with n <= 1e4 and 1 <= a_i <= 1e4. --max: 20 tests of
n = 1e4 multiples of 3, so every window sum is composite, odd or even, and all n^2 / 2 are tried.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
t = 20 if MAX else rng.randint(1, 5)
print(t)
for _ in range(t):
    n = 10**4 if MAX else rng.randint(1, max(1, min(10**4, int(sys.argv[2]))))
    kind = "three" if MAX else rng.choice(("three", "even", "small", "big", "mixed"))
    if kind == "three":
        a = [3 * rng.randint(1, 3333) for _ in range(n)]
    elif kind == "even":
        a = [2 * rng.randint(1, 5000) for _ in range(n)]
    elif kind == "small":
        a = [rng.randint(1, 3) for _ in range(n)]
    elif kind == "big":
        a = [rng.randint(9000, 10**4) for _ in range(n)]
    else:
        a = [2 * rng.randint(1, 5000) if rng.random() < 0.9 else rng.randint(1, 10**4) for _ in range(n)]
    print(n, *a)
