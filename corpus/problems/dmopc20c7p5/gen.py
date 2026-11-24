"""gen.py seed size: DMOPC '20 C7 P5. 1 <= N <= 500, 0 <= M <= 500, values 0..=1e9.
size >= 500 gives N = M = 500 with fees and tolls of similar size, so the cut is not trivial."""
import random, sys
if sys.argv[1:] == ["--max"]:
    # N = M = 500 with fees and tolls up to 1e9 of similar size, so the cut is not trivial
    sys.argv[1:] = ['0', '500']
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 500
n = 500 if big else rng.randint(1, max(1, min(size, 8)))
m = 500 if big else rng.randint(0, max(1, min(size, 10)))
hi = 10 ** 9 if big or rng.random() < 0.3 else 10
print(n, m)
print(*[rng.randint(0, hi) for _ in range(n)])
for _ in range(m):
    print(rng.randint(1, n), rng.randint(1, n), min(10 ** 9, rng.randint(0, 2 * hi)))
