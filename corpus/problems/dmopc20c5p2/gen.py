"""gen.py seed size: DMOPC '20 Contest 5 P2. 1 <= N, M <= 1e6.
size >= 1e6 gives N, M near 1e6 by seed (coprime, equal, one a multiple of the other)."""
import random, sys
if sys.argv[1:] == ["--max"]: sys.argv[1:] = ["4", "1000000"]  # the hardest big case
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size >= 10 ** 6:
    n, m = [(999_999, 1_000_000), (10 ** 6, 10 ** 6), (10 ** 6, 500_000), (1, 10 ** 6)][seed % 4]
else:
    hi = max(1, min(size, 60))
    n, m = rng.randint(1, hi), rng.randint(1, hi)
    if rng.random() < 0.3: m = n * rng.randint(1, 3)
print(n, m)
