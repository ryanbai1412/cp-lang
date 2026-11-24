"""gen.py seed size: DMOPC '20 C7 P1. 1 <= n <= 5e5, masses 1..=1e9.
size >= 5e5 gives n = 5e5: by seed, distinct masses, few masses, or one mass on most weights."""
import random, sys
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 5 * 10 ** 5
n = 5 * 10 ** 5 if big else rng.randint(1, max(1, min(size, 8)))
shape = seed % 3 if big else rng.randrange(3)
if shape == 0: a = [rng.randint(1, 10 ** 9) for _ in range(2 * n)]
elif shape == 1: a = [rng.randint(1, 3) for _ in range(2 * n)]
else: a = [7 if rng.random() < 0.7 else rng.randint(1, 10 ** 9) for _ in range(2 * n)]
print(n); print(*a)
