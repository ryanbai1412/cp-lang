"""noip19p4 generator: n <= 100 methods by m <= 2000 ingredients, 0 <= a_ij < 998244353."""
import random
import sys

P = 998244353


def emit(rng, n, m, hi):
    print(n, m)
    for _ in range(n):
        print(*(rng.randint(0, hi) for _ in range(m)))


if sys.argv[1] == "--max":
    emit(random.Random(0), 100, 2000, P - 1)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
emit(rng, rng.randint(1, max(1, min(100, size))), rng.randint(1, max(1, min(2000, size))), rng.choice([1, 3, P - 1]))
