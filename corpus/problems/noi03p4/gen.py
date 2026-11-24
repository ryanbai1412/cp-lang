"""noi03p4 generator: `N N-1`, then a tree of N >= 3 houses with street times 1..=1e9."""
import random
import sys

if sys.argv[1] == "--max":
    rng = random.Random(0)
    n = 200_000
    print(n, n - 1)
    # A long caterpillar: deep walks and many near-tied detours.
    for v in range(2, n + 1):
        u = v - 1 if v <= n // 2 else rng.randint(1, n // 2)
        print(u, v, rng.randint(10**9 - 1000, 10**9))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(3, max(3, size))
w = rng.choice([1, 10, 10**9])
perm = list(range(1, n + 1))
rng.shuffle(perm)
print(n, n - 1)
for i in range(1, n):
    j = rng.choice([i - 1, rng.randrange(i)])
    print(perm[i], perm[j], rng.randint(1, w))
